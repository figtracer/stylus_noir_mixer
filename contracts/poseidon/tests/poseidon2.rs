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
        "29f5cdf6eb8e04fd0f33b56f3c6bac311204572ba750cbeae363d2c6321dbb23"
    ));
    let input_b = U256::from_be_slice(&hex!(
        "1b1d665fb1656592e865b53c2e09b2d88000cb64bb86f09ccf85b75378b7fdf1"
    ));

    let PoseidonAbi::hashReturn { hash } = contract.hash([input_a, input_b]).call().await?;

    let expected = U256::from_be_slice(&hex!(
        "0fc5f23f3d0c1c4d0c2723d576b84e883e1c4dad6325104f18572c1057017eeb"
    ));

    assert_eq!(hash, expected);

    Ok(())
}
