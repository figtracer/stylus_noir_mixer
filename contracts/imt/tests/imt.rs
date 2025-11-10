#![cfg(feature = "e2e")]

use alloy_primitives::{hex, uint, FixedBytes};
use e2e::{constructor, Account};
use eyre::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::abi::IMTAbi;
mod abi;

#[e2e::test]
async fn imt_insert_works(alice: Account) -> Result<()> {
    /* deploy poseidon */
    let poseidon_rcpt = deploy_poseidon(&alice).await?;
    let poseidon_addr = poseidon_rcpt.contract_address;
    println!("poseidon deployed at: {poseidon_addr:?}");

    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(5_U256), poseidon_addr))
        .deploy()
        .await?
        .contract_address;
    println!(
        "IMT deployed at: {contract_addr:?} with constructor: (uint!(5_U256), {poseidon_addr:?})"
    );
    let contract = IMTAbi::new(contract_addr, &alice.wallet);

    /* generate commitment */
    let commitment = generate_commitment_from_ts()?;
    println!(
        "commitment: 0x{}",
        alloy::hex::encode(commitment.as_slice()),
    );

    /* insert commitment */
    let IMTAbi::insertReturn { _0: index } = contract.insert(commitment).call().await?;
    println!("insert tx succeeded. nextLeafIndex: {index:?}");
    Ok(())
}

#[e2e::test]
async fn imt_zeros_match_constants(alice: Account) -> Result<()> {
    /* deploy poseidon */
    let poseidon_rcpt = deploy_poseidon(&alice).await?;
    let poseidon_addr = poseidon_rcpt.contract_address;
    println!("poseidon deployed at: {poseidon_addr:?}");

    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(5_U256), poseidon_addr))
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
    /* deploy poseidon */
    let poseidon_rcpt = deploy_poseidon(&alice).await?;
    let poseidon_addr = poseidon_rcpt.contract_address;
    println!("poseidon deployed at: {poseidon_addr:?}");

    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(5_U256), poseidon_addr))
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
async fn deploy_poseidon(alice: &Account) -> Result<e2e::Receipt> {
    let poseidon_wasm = poseidon_wasm_path()?;
    let poseidon_rcpt = alice.as_deployer().deploy_wasm(&poseidon_wasm).await?;
    Ok(poseidon_rcpt)
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

fn poseidon_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "openzeppelin_poseidon.wasm"; // cargo turns '-' into '_'
    let path = root
        .join("contracts/poseidon/target/wasm32-unknown-unknown/release")
        .join(file);
    if !path.exists() {
        return Err(eyre::eyre!(
            "poseidon wasm not found at {}. run `npm run check:checks` first to build it.",
            path.display()
        ));
    }
    Ok(path)
}

fn generate_commitment_from_ts() -> eyre::Result<FixedBytes<32>> {
    let root = repo_root();
    let script = root.join("scripts/js/generateCommitment.ts");

    let output = Command::new("npx")
        .args(["tsx", script.to_str().expect("valid script path")])
        .current_dir(&root)
        .output()?;

    if !output.status.success() {
        return Err(eyre::eyre!(
            "commitment script failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let s = String::from_utf8(output.stdout)?;
    let s = s.strip_prefix("0x").unwrap_or(&s);

    let bytes = alloy::hex::decode(s)?;
    if bytes.len() != 96 {
        return Err(eyre::eyre!(
            "unexpected commitment payload size: {} bytes",
            bytes.len()
        ));
    }

    let to_fb32 = |chunk: &[u8]| {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(chunk);
        FixedBytes::<32>::from(arr)
    };

    Ok(to_fb32(&bytes[0..32]))
}
