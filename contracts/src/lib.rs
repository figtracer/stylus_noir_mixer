#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

/* modules and imports */
pub mod poseidon2;
pub mod imt;
pub mod errors;

use crate::imt::IncrementalMerkleTree;
use crate::errors::ContractErrors;
use stylus_sdk::{
                 alloy_primitives::{U256, U32, uint},
                 alloy_sol_types::sol,
                 prelude::*,
                 storage::{StorageBool, StorageGuard},
                };

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

sol_storage! {
    #[entrypoint] 
    pub struct Mixer {
        #[borrow]
        IncrementalMerkleTree imt;
        mapping(bytes32 => bool) commitments;
    }
}

/* events */
sol! {
    /* events */
    event CommitmentInserted(uint32 indexed index);
}

#[public]
#[inherit(IncrementalMerkleTree)]
impl Mixer {
    /* initializes the imt with the given depth */
    #[constructor]
    pub fn init(&mut self, depth: U32) {
        let _ = self.imt.init(depth);
    }

    #[payable]
    pub fn deposit(&mut self, commitment: alloy_primitives::FixedBytes<32>) -> Result<(), ContractErrors> {
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

        let inserted_index: U32 = self.imt.insert(commitment)?;

        /* convert the inserted index to a u32, otherwise we can't log it */
        let idx_u32: u32 = u32::from_be_bytes(inserted_index.to_be_bytes::<4>());
        log(self.vm(), CommitmentInserted { index: idx_u32 });
        Ok(())
    }
}
