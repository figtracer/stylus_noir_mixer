#![cfg(feature = "e2e")]

use alloy_primitives::{uint, Address, FixedBytes, U256};
use e2e::{constructor, Account};
use eyre::Result;
use std::path::PathBuf;
use std::process::Command;

mod abi;
use abi::MixerAbi;

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

#[e2e::test]
async fn mixer_deposit_works(alice: Account) -> Result<()> {
    /* deploy poseidon */
    let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
    /* deploy imt */
    let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
    /* deploy verifier */
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    /* deploy mixer */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let commitment = generate_commitment_from_ts()?;
    println!(
        "commitment: 0x{}",
        alloy::hex::encode(commitment.as_slice()),
    );

    mixer.deposit(commitment).value(DENOMINATION).call().await?;
    // TODO: assert
    Ok(())
}

#[e2e::test]
async fn mixer_deposit_rejects_invalid_denomination(alice: Account) -> Result<()> {
    /* deploy poseidon */
    let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
    /* deploy imt */
    let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
    /* deploy verifier */
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    /* deploy mixer */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let commitment = generate_commitment_from_ts()?;
    println!(
        "commitment: 0x{}",
        alloy::hex::encode(commitment.as_slice()),
    );

    /* call deposit with zero value -> expect revert */
    mixer.deposit(commitment).value(U256::ZERO).call().await?;
    // TODO: assert
    Ok(())
}

#[e2e::test]
async fn mixer_deposit_rejects_duplicate_commitment(alice: Account) -> Result<()> {
    /* deploy poseidon */
    let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
    /* deploy imt */
    let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
    /* deploy verifier */
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    /* deploy mixer */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let commitment = generate_commitment_from_ts()?;
    println!(
        "commitment: 0x{}",
        alloy::hex::encode(commitment.as_slice()),
    );

    mixer.deposit(commitment).value(DENOMINATION).call().await?;
    mixer.deposit(commitment).value(DENOMINATION).call().await?;
    // TODO: assert
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

async fn deploy_imt(alice: &Account, poseidon_addr: Address) -> Result<e2e::Receipt> {
    let imt_wasm = imt_wasm_path()?;
    let imt_rcpt = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(20_U256), poseidon_addr))
        .deploy_wasm(&imt_wasm)
        .await?;
    Ok(imt_rcpt)
}

async fn deploy_mixer(
    alice: &Account,
    verifier_addr: Address,
    poseidon_addr: Address,
    imt_addr: Address,
) -> Result<e2e::Receipt> {
    let mixer_wasm = mixer_wasm_path()?;
    let mixer_rcpt = alice
        .as_deployer()
        .with_constructor(constructor!(verifier_addr, poseidon_addr, imt_addr))
        .deploy_wasm(&mixer_wasm)
        .await?;
    Ok(mixer_rcpt)
}

fn mixer_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "stylus_mixer.wasm";
    let path = root
        .join("contracts/mixer/target/wasm32-unknown-unknown/release")
        .join(file);
    if !path.exists() {
        return Err(eyre::eyre!(
            "mixer wasm not found at {}. run `npm run check:checks` first to build it.",
            path.display()
        ));
    }
    Ok(path)
}

/* get WASM paths */
fn poseidon_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "openzeppelin_poseidon.wasm";
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

fn imt_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "stylus_imt.wasm";
    let path = root
        .join("contracts/imt/target/wasm32-unknown-unknown/release")
        .join(file);
    if !path.exists() {
        return Err(eyre::eyre!(
            "imt wasm not found at {}. run `npm run check:checks` first to build it.",
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

fn repo_root() -> PathBuf {
    /* contracts/imt -> project root */
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}
