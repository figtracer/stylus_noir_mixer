#![cfg(feature = "e2e")]

use alloy_primitives::{hex, uint, FixedBytes};
use e2e::{constructor, Account};
use eyre::Result;
use serde::Deserialize;
use std::{path::PathBuf, process::Command};

use crate::abi::IMTAbi;
mod abi;

#[e2e::test]
async fn imt_insert_works(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(15_U256)))
        .deploy()
        .await?
        .contract_address;
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    /* generate commitment */
    let (commitment, nullifier, secret) = generate_commitment()?;

    /* insert commitment */
    let IMTAbi::insertReturn { _0: index } = contract.insert(commitment).call().await?;
    assert_eq!(index, 0);
    Ok(())
}

#[e2e::test]
async fn imt_zeros_match_constants(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(15_U256)))
        .deploy()
        .await?
        .contract_address;
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    let IMTAbi::zerosReturn { z: z0 } = contract.zeros(uint!(0_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z1 } = contract.zeros(uint!(1_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z2 } = contract.zeros(uint!(2_U256)).call().await?;
    let IMTAbi::zerosReturn { z: z10 } = contract.zeros(uint!(10_U256)).call().await?;

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
    let e10 = {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hex!(
            "26c093f627ffb8a25ab933cf64dd4f29dae2b103b48db3bf619f0dc39b298222"
        ));
        FixedBytes::<32>::from(arr)
    };

    assert_eq!(z0, e0);
    assert_eq!(z1, e1);
    assert_eq!(z2, e2);
    assert_eq!(z10, e10);

    Ok(())
}

#[e2e::test]
async fn imt_is_known_root_zero_is_false(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(15_U256)))
        .deploy()
        .await?
        .contract_address;
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    let IMTAbi::isKnownRootReturn { known } =
        contract.isKnownRoot(FixedBytes::<32>::ZERO).call().await?;
    assert!(!known);

    Ok(())
}

/* ======================================================================
 *                               INTERNAL HELPERS
 * ====================================================================== */
#[derive(Deserialize)]
struct CommitmentResponse {
    commitment: String,
    nullifier: String,
    secret: String,
}

fn generate_commitment() -> eyre::Result<(FixedBytes<32>, FixedBytes<32>, FixedBytes<32>)> {
    let root = repo_root();
    let script = root.join("scripts/js/generateCommitment.ts");
    let output = Command::new("npx")
        .args(["tsx", script.to_str().expect("valid script path")])
        .current_dir(&root)
        .output()?;

    let s = String::from_utf8(output.stdout)?;
    let resp: CommitmentResponse = serde_json::from_str(&s)?;
    let commitment = hex_to_fixed_bytes(&resp.commitment)?;
    let nullifier = hex_to_fixed_bytes(&resp.nullifier)?;
    let secret = hex_to_fixed_bytes(&resp.secret)?;

    Ok((commitment, nullifier, secret))
}

fn hex_to_fixed_bytes(s: &str) -> eyre::Result<FixedBytes<32>> {
    let bytes = alloy::hex::decode(s)?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| eyre::eyre!("expected 32-byte hex string"))?;
    Ok(FixedBytes::<32>::from(arr))
}

fn repo_root() -> PathBuf {
    /* contracts/imt -> project root */
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}
