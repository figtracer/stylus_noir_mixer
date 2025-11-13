#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

mod params;
mod sponge;

use alloy_primitives::FixedBytes;
use openzeppelin_crypto::{
    arithmetic::{uint::U256, BigInteger},
    field::instance::FpBN256,
};

#[cfg(feature = "contract")]
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

/* =====================================================================
 *                               Helpers
 * ====================================================================== */
pub fn hash_two_fixed_bytes(left: FixedBytes<32>, right: FixedBytes<32>) -> FixedBytes<32> {
    let inputs = [fixed_bytes_to_fp(left), fixed_bytes_to_fp(right)];
    fp_to_fixed_bytes(sponge::hash(&inputs, inputs.len(), false))
}

fn fixed_bytes_to_fp(value: FixedBytes<32>) -> FpBN256 {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(value.as_slice());
    bytes.reverse();
    FpBN256::from_bigint(U256::from_le_slice(&bytes))
}

fn fp_to_fixed_bytes(value: FpBN256) -> FixedBytes<32> {
    let mut le_bytes = value.into_bigint().into_bytes_le();
    le_bytes.resize(32, 0);
    let mut be_bytes = [0u8; 32];
    for (i, byte) in le_bytes.iter().enumerate() {
        be_bytes[31 - i] = *byte;
    }
    FixedBytes::<32>::from(be_bytes)
}
