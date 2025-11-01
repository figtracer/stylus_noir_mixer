#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

/* modules and imports */
pub mod poseidon2;
pub mod imt;

use crate::imt::IncrementalMerkleTree;
use stylus_sdk::{
                 alloy_primitives::{U256, uint},
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
    event CommitmentInserted(uint256 indexed index);
}

#[public]
#[inherit(IncrementalMerkleTree)]
impl Mixer {
    #[payable]
    pub fn deposit(&mut self, commitment: alloy_primitives::FixedBytes<32>) {
        /* check if commitment is already present */
        let guard: StorageGuard<StorageBool> = self.commitments.getter(commitment);
        if guard.get() {
            return;
        }

        /* check if amount sent is the same as the denomination value for the mixer */
        let amount = self.vm().msg_value();
        if amount < DENOMINATION {
            return;
        }

        self.commitments.insert(commitment, true);

        // let inserted_index: U256 = self.imt.insert(commitment);

        // log(self.vm(), CommitmentInserted { index: inserted_index });    
    }
}
