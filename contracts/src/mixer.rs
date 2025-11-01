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
    event CommitmentInserted(uint32 indexed index);
}

#[public]
#[inherit(IncrementalMerkleTree)]
impl Mixer {
    /* initializes the imt with the given depth */
    #[constructor]
    pub fn init(&mut self, depth: U32) {
        self.imt.init(depth);
    }

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

        let inserted_index: U32 = self.imt.insert(commitment);

        /* convert the inserted index to a u32 */
        let idx_u32: u32 = u32::from_be_bytes(inserted_index.to_be_bytes::<4>());
        log(self.vm(), CommitmentInserted { index: idx_u32 });    
    }
}
