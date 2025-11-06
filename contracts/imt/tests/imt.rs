#![cfg(feature = "e2e")]

use alloy_primitives::{hex, uint, FixedBytes};
use e2e::{constructor, Account};
use eyre::Result;

use crate::abi::IMTAbi;
mod abi;

#[e2e::test]
async fn imt_zeros_match_constants(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(5_U256)))
        .deploy()
        .await?
        .contract_address;
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    let IMTAbi::zerosReturn { z: z0 } = contract.zeros(uint!(0_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z1 } = contract.zeros(uint!(1_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z2 } = contract.zeros(uint!(2_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z31 } = contract.zeros(uint!(31_U256)).call().await?;

    let e0 = {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hex!(
            "168db4aa1d4e4bf2ee46eb882e1c38a7de1a4da47e17b207a5494a14605ae38e"
        ));
        FixedBytes::<32>::from(arr)
    };
    let e1 = {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hex!(
            "257a568bdc9cc663b2cf123f7d7b6c5eedd5a312d2792305352e09f1733a56b5"
        ));
        FixedBytes::<32>::from(arr)
    };
    let e2 = {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hex!(
            "25b9b4ff326c7783ce7a3ae1503dce4552211bdfb510808e215f4227da087023"
        ));
        FixedBytes::<32>::from(arr)
    };
    let e31 = {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hex!(
            "13b6403089d691e83af7392d8e9bddd76e83d8204b2475fc6c60679bd338dea8"
        ));
        FixedBytes::<32>::from(arr)
    };

    assert_eq!(z0, e0);
    assert_eq!(z1, e1);
    assert_eq!(z2, e2);
    assert_eq!(z31, e31);

    Ok(())
}

#[e2e::test]
async fn imt_is_known_root_zero_is_false(alice: Account) -> Result<()> {
    // Deploy IMT with any valid depth
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(5_U256)))
        .deploy()
        .await?
        .contract_address;
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    let IMTAbi::isKnownRootReturn { known } =
        contract.isKnownRoot(FixedBytes::<32>::ZERO).call().await?;
    assert!(!known);

    Ok(())
}
