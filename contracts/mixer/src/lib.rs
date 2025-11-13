#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

pub mod interface;

use crate::interface::VerifierInterface;
use stylus_common::errors::MixerErrors;
use stylus_imt::interface::IMTInterface;
use stylus_sdk::{
    abi::Bytes as AbiBytes,
    alloy_primitives::{uint, Address, Bytes as AlloyBytes, FixedBytes, U256},
    alloy_sol_types::sol,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageGuard, StorageMap},
};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

sol! {
    event Deposit(bytes32 indexed commitment, uint32 index, uint256 timestamp);
    event Withdrawal(address indexed recipient, bytes32 indexed nullifier_hash);
}

#[entrypoint]
#[storage]
pub struct Mixer {
    commitments: StorageMap<FixedBytes<32>, StorageBool>,
    nullifier_hashes: StorageMap<FixedBytes<32>, StorageBool>,
    imt: StorageAddress,
    verifier: StorageAddress,
}

/* ======================================================================
 *                               Contract
 * ====================================================================== */
#[public]
impl Mixer {
    #[constructor]
    fn initialize(&mut self, verifier: Address, imt: Address) -> Result<(), MixerErrors> {
        self.verifier.set(verifier);
        self.imt.set(imt);
        Ok(())
    }

    #[payable]
    fn deposit(&mut self, commitment: FixedBytes<32>) -> Result<(), MixerErrors> {
        /* check if commitment is already present */
        let guard: StorageGuard<StorageBool> = self.commitments.getter(commitment);
        if guard.get() {
            return Err(MixerErrors::commitment_already_exists());
        }

        /* check if amount sent is the same as the denomination value for the mixer */
        let amount = self.vm().msg_value();
        if amount < DENOMINATION {
            return Err(MixerErrors::invalid_denomination());
        }

        self.commitments.insert(commitment, true);

        let inserted_index = IMTInterface::new(self.imt.get())
            .insert(&mut *self, commitment)
            .expect("insert call failed");

        log(
            self.vm(),
            Deposit {
                commitment: commitment,
                index: inserted_index,
                timestamp: U256::from(self.vm().block_timestamp()),
            },
        );
        Ok(())
    }

    fn withdraw(
        &mut self,
        proof: AbiBytes,
        root: FixedBytes<32>,
        nullifier_hash: FixedBytes<32>,
        recipient: Address,
    ) -> Result<(), MixerErrors> {
        /* check if nullifier hash has already been used */
        if self.nullifier_hashes.getter(nullifier_hash).get() {
            return Err(MixerErrors::nullifier_hash_already_used());
        }

        /* check if root is known */
        let known = IMTInterface::new(self.imt.get())
            .is_known_root(&mut *self, root)
            .expect("isKnownRoot call failed");
        if !known {
            return Err(MixerErrors::invalid_root());
        }

        let bytes_recipient: FixedBytes<32> = recipient.into_word();

        /* prepare public inputs for the verifier */
        let mut public_inputs: Vec<FixedBytes<32>> = Vec::with_capacity(3);
        public_inputs.push(root);
        public_inputs.push(nullifier_hash);
        public_inputs.push(bytes_recipient);

        /* verify proof */
        let verified = VerifierInterface::new(self.verifier.get())
            .verify(
                &mut *self,
                AlloyBytes::copy_from_slice(proof.as_slice()),
                public_inputs,
            )
            .expect("verify call failed");
        if !verified {
            return Err(MixerErrors::invalid_proof());
        }

        /* insert nullifier hash */
        self.nullifier_hashes.insert(nullifier_hash, true);

        /* transfer funds to recipient */
        self.vm()
            .transfer_eth(recipient, DENOMINATION)
            .map_err(|_| MixerErrors::invalid_denomination())?;

        log(
            self.vm(),
            Withdrawal {
                recipient,
                nullifier_hash,
            },
        );
        Ok(())
    }
}
