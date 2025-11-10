#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

pub mod interface;

use crate::interface::IVerifier;
use stylus_common::errors::ContractErrors;
use stylus_imt::interface::IIMT;

use stylus_sdk::{
    abi::Bytes as AbiBytes,
    alloy_primitives::{uint, Address, Bytes as AlloyBytes, FixedBytes, U256},
    alloy_sol_types::sol,
    call::Call,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageGuard, StorageMap},
};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

sol! {
    event CommitmentInserted(uint32 indexed index);
    event Withdrawal(address indexed recipient, bytes32 indexed nullifier_hash);
}

#[entrypoint]
#[storage]
pub struct Mixer {
    commitments: StorageMap<FixedBytes<32>, StorageBool>,
    nullifier_hashes: StorageMap<FixedBytes<32>, StorageBool>,
    imt: StorageAddress,
    verifier: StorageAddress,
    hasher: StorageAddress,
}

#[public]
impl Mixer {
    #[constructor]
    fn initialize(
        &mut self,
        verifier: Address,
        hasher: Address,
        imt: Address,
    ) -> Result<(), ContractErrors> {
        self.verifier.set(verifier);
        self.hasher.set(hasher);
        self.imt.set(imt);
        Ok(())
    }

    #[payable]
    fn deposit(&mut self, commitment: FixedBytes<32>) -> Result<(), ContractErrors> {
        /* check if commitment is already present */
        let guard: StorageGuard<StorageBool> = self.commitments.getter(commitment);
        if guard.get() {
            return Err(ContractErrors::commitment_already_exists());
        }

        /* check if amount sent is the same as the denomination value for the mixer */
        let amount = self.vm().msg_value();
        if amount < DENOMINATION {
            return Err(ContractErrors::invalid_denomination());
        }

        self.commitments.insert(commitment, true);

        let inserted_index = IIMT::new(self.imt.get())
            .insert(Call::new(), commitment)
            .expect("insert call failed");

        log(
            self.vm(),
            CommitmentInserted {
                index: inserted_index,
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
    ) -> Result<(), ContractErrors> {
        /* check if nullifier hash has already been used */
        if self.nullifier_hashes.getter(nullifier_hash).get() {
            return Err(ContractErrors::nullifier_hash_already_used());
        }

        /* check if root is known */
        let known = IIMT::new(self.imt.get())
            .is_known_root(Call::new(), root)
            .expect("isKnownRoot call failed");
        if !known {
            return Err(ContractErrors::invalid_root());
        }

        let bytes_recipient: FixedBytes<32> = recipient.into_word();

        /* prepare public inputs for the verifier */
        let mut public_inputs: Vec<FixedBytes<32>> = Vec::with_capacity(3);
        public_inputs.push(root);
        public_inputs.push(nullifier_hash);
        public_inputs.push(bytes_recipient);

        /* verify proof */
        let verified = IVerifier::new(self.verifier.get())
            .verify(
                Call::new(),
                AlloyBytes::copy_from_slice(proof.as_slice()),
                public_inputs,
            )
            .expect("verify call failed");
        if !verified {
            return Err(ContractErrors::invalid_proof());
        }

        /* insert nullifier hash */
        self.nullifier_hashes.insert(nullifier_hash, true);

        /* transfer funds to recipient */
        self.vm()
            .transfer_eth(recipient, DENOMINATION)
            .map_err(|_| ContractErrors::invalid_denomination())?;

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
