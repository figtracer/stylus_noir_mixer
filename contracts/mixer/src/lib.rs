#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

/* modules and imports */
use stylus_common::errors::ContractErrors;
use stylus_imt::IMT;

use stylus_sdk::{
    alloy_primitives::{uint, Address, U256, U32},
    alloy_sol_types::sol,
    prelude::*,
    storage::{StorageBool, StorageGuard},
};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

sol! {
    /* events */
    event CommitmentInserted(uint32 indexed index);
}

sol_interface! {
    /* interfaces */
    interface IIMT {
        function insert(bytes32 commitment) external returns (uint32);
    }
}

sol_storage! {
    #[entrypoint]
    pub struct Mixer {
        mapping(bytes32 => bool) commitments;
        address imt;
    }
}

#[public]
impl Mixer {
    /* todo: add permission check */
    pub fn set_imt(&mut self, addr: Address) -> Result<(), ContractErrors> {
        self.imt.set(addr);
        Ok(())
    }

    #[payable]
    pub fn deposit<S: TopLevelStorage>(
        storage: &mut S,
        commitment: alloy_primitives::FixedBytes<32>,
    ) -> Result<(), ContractErrors> {
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

        let inserted_index: U32 = {
            let imt_address = self.imt.get();
            let imt = IIMT::new(imt_address);
            imt.insert(storage, commitment).expect("insert call failed")
        };

        /* convert the inserted index to a u32, otherwise we can't log it */
        let idx_u32: u32 = u32::from_be_bytes(inserted_index.to_be_bytes::<4>());
        log(self.vm(), CommitmentInserted { index: idx_u32 });
        Ok(())
    }
}
