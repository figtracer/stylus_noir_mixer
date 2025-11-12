#![cfg(feature = "e2e")]

use alloy_primitives::{hex, uint, U256};
use e2e::Account;
use eyre::Result;

use crate::abi::PoseidonAbi;
mod abi;

#[e2e::test]
async fn poseidon_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PoseidonAbi::new(contract_addr, &alice.wallet);

    let PoseidonAbi::hashReturn { hash } = contract
        .hash([uint!(123_U256), uint!(123456_U256)])
        .call()
        .await?;

    let expected = U256::from_be_slice(&hex!(
        "1f24fc186957171704ab4ddf424d2830a3f5d04910752a162cd93487ebdc634d"
    ));

    assert_eq!(hash, expected);

    Ok(())
}

#[e2e::test]
async fn poseidon_known_vector(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PoseidonAbi::new(contract_addr, &alice.wallet);

    let input_a = U256::from_be_slice(&hex!(
        "1f8fb4ad3f03c2e36e1fcf77a43e41b55f01c37231981130f687b0019df78374"
    ));
    let input_b = U256::from_be_slice(&hex!(
        "080104bc5c9cc4a6c922cf39f3a7f3e8820d988904094e3d5087c0bc3e93e3bc"
    ));

    let PoseidonAbi::hashReturn { hash } = contract.hash([input_a, input_b]).call().await?;

    let expected = U256::from_be_slice(&hex!(
        "083d10323077fed15f77b82c26a7f28ae8ce785a19716a26e2c96d695a8effae"
    ));

    assert_eq!(hash, expected);

    Ok(())
}
