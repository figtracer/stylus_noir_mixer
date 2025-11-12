#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

pub mod interface;
mod params;
mod sponge;

use openzeppelin_crypto::{arithmetic::uint::U256, field::instance::FpBN256};
use stylus_sdk::prelude::*;

#[cfg(feature = "contract")]
#[entrypoint]
#[storage]
struct Poseidon;

/* =====================================================================
 *                               Contract
 * ====================================================================== */
#[cfg(feature = "contract")]
#[public]
impl Poseidon {
    fn hash(&self, inputs: [alloy_primitives::U256; 2]) -> alloy_primitives::U256 {
        let fp_inputs = inputs.map(|input| FpBN256::from_bigint(U256::from(input)));
        let hash = sponge::hash(&fp_inputs, fp_inputs.len(), false);
        hash.into_bigint().into()
    }
}
