#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

/* modules and imports */
use stylus_common::errors::ContractErrors;
use stylus_imt::interface::IIMT;
use stylus_sdk::{
    alloy_primitives::{uint, Address, FixedBytes, U256},
    alloy_sol_types::sol,
    call::Call,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageGuard, StorageMap},
};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

sol! {
    event CommitmentInserted(uint32 indexed index);
}

#[entrypoint]
#[storage]
pub struct Mixer {
    commitments: StorageMap<FixedBytes<32>, StorageBool>,
    imt: StorageAddress,
}

#[public]
impl Mixer {
    #[constructor]
    fn initialize(&mut self, imt: Address) -> Result<(), ContractErrors> {
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

        let imt = IIMT::new(self.imt.get());
        let inserted_index = imt
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
}
