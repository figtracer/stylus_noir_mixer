#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

mod incremental_merkle_tree;
pub mod poseidon2;
use crate::incremental_merkle_tree::IncrementalMerkleTree;

use alloc::vec::Vec;
use stylus_sdk::{alloy_primitives::U256};
use stylus_sdk::{alloy_primitives::uint};
use stylus_sdk::{alloy_sol_types::sol};
use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageBool, StorageGuard};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

/* contract storage */
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
