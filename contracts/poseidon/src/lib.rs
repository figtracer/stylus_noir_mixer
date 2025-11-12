#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

pub mod interface;
mod params;

use openzeppelin_crypto::{arithmetic::uint::U256, field::instance::FpBN256, poseidon2::Poseidon2};
use stylus_sdk::prelude::*;

use crate::params::StylusBN256Params;

#[cfg(feature = "contract")]
#[entrypoint]
#[storage]
struct Poseidon;

/* ======================================================================
 *                               Contract
 * ====================================================================== */
#[cfg(feature = "contract")]
#[public]
impl Poseidon {
    fn hash(&self, inputs: [alloy_primitives::U256; 2]) -> alloy_primitives::U256 {
        let mut hasher = Poseidon2::<StylusBN256Params, FpBN256>::new();

        for input in &inputs {
            let fp = FpBN256::from_bigint(U256::from(*input));
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        hash.into_bigint().into()
    }
}
