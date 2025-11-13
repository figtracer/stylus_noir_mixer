#![cfg(feature = "e2e")]

use alloy::{eips::BlockId, providers::Provider, rpc::types::BlockTransactionsKind};
use alloy_primitives::{uint, Address, FixedBytes, U256};
use e2e::{constructor, receipt, send, Account, Revert};
use eyre::{Result, WrapErr};
use serde::Deserialize;
use std::str::FromStr;
use std::{path::PathBuf, process::Command};

mod abi;
use abi::{IMTAbi, MixerAbi};

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

/* ======================================================================
 *                      generate commmitment and proof
 * ====================================================================== */
#[e2e::test]
async fn generate_commitment_and_proof_works(alice: Account) -> Result<()> {
    let (commitment, nullifier, secret) = generate_commitment()?;
    println!(
        "commitment: 0x{}",
        alloy::hex::encode(commitment.as_slice())
    );
    println!("nullifier: 0x{}", alloy::hex::encode(nullifier.as_slice()));
    println!("secret: 0x{}", alloy::hex::encode(secret.as_slice()));
    let leaves = vec![commitment];
    let recipient = alice.address();
    let (proof, public_inputs) = generate_proof(nullifier, secret, recipient, leaves)?;
    println!("proof: 0x{}", alloy::hex::encode(proof.as_slice()));
    println!("public_inputs: {:?}", public_inputs);
    Ok(())
}
/* ======================================================================
 *                               deposit()
 * ====================================================================== */
#[e2e::test]
async fn mixer_deposit_works(alice: Account) -> Result<()> {
    let imt_addr = deploy_imt(&alice).await?.contract_address;
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let (commitment, _nullifier, _secret) = generate_commitment()?;

    let rcpt = receipt!(mixer.deposit(commitment).value(DENOMINATION))?;

    /* record timestamp right after the deposit */
    let timestamp = U256::from(block_timestamp(&alice).await?);

    let raw_log = rcpt.inner.as_receipt().unwrap().logs.first().unwrap();
    let decoded = raw_log
        .log_decode::<MixerAbi::Deposit>()
        .expect("decode deposit event");

    let event = &decoded.inner.data;
    assert_eq!(event.commitment, commitment);
    assert_eq!(event.timestamp, timestamp);
    assert_eq!(event.index, 0u32);
    Ok(())
}

#[e2e::test]
async fn mixer_deposit_rejects_invalid_denomination(alice: Account) -> Result<()> {
    let imt_addr = deploy_imt(&alice).await?.contract_address;
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let (commitment, _nullifier, _secret) = generate_commitment()?;

    /* call deposit with zero value -> expect revert */
    let err = send!(mixer.deposit(commitment).value(U256::ZERO)).expect_err("should revert");
    assert!(err.reverted_with(MixerAbi::InvalidDenomination {}));
    Ok(())
}

#[e2e::test]
async fn mixer_deposit_rejects_duplicate_commitment(alice: Account) -> Result<()> {
    let imt_addr = deploy_imt(&alice).await?.contract_address;
    let verifier_addr = Address::ZERO; /* not needed for deposit path */
    let mixer_addr = deploy_mixer(&alice, verifier_addr, imt_addr)
        .await?
        .contract_address;
    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

    /* generate commitment */
    let (commitment, _nullifier, _secret) = generate_commitment()?;

    receipt!(mixer.deposit(commitment).value(DENOMINATION))?;
    let err = send!(mixer.deposit(commitment).value(DENOMINATION)).expect_err("should revert");
    assert!(err.reverted_with(MixerAbi::CommitmentAlreadyExists {}));
    Ok(())
}

/* ======================================================================
 *                               withdraw()
 * ====================================================================== */
#[e2e::test]
async fn mixer_withdraw_works(alice: Account) -> Result<()> {
    let imt_addr = deploy_imt(&alice).await?.contract_address;
    println!("IMT: {imt_addr:?}\n");
    let verifier_addr = deploy_verifier()?;
    println!("VERIFIER: {verifier_addr:?}\n");
    let mixer_addr = deploy_mixer(&alice, verifier_addr, imt_addr)
        .await?
        .contract_address;
    println!("MIXER: {mixer_addr:?}\n");

    let mixer = MixerAbi::new(mixer_addr, &alice.wallet);
    let imt = IMTAbi::new(imt_addr, &alice.wallet);

    /* generate commitment */
    let (commitment, nullifier, secret) = generate_commitment()?;
    println!("COMMITMENT: {commitment:?}\n");
    println!("NULLIFIER: {nullifier:?}\n");
    println!("SECRET: {secret:?}\n");

    let rcpt = receipt!(mixer.deposit(commitment).value(DENOMINATION))?;

    /* this is cheating a little bit because we're not actually using the merkle tree */
    let leaves = vec![commitment];

    /* generate proof */
    let (proof, public_inputs) = generate_proof(nullifier, secret, alice.address(), leaves)?;

    let IMTAbi::isKnownRootReturn { known } = imt.isKnownRoot(public_inputs[0]).call().await?;
    assert!(known, "proof root not found in IMT");

    receipt!(mixer.withdraw(
        proof.into(),
        public_inputs[0],
        public_inputs[1],
        Address::from_word(public_inputs[2])
    ))?;
    Ok(())
}
/* ======================================================================
 *                               INTERNAL HELPERS
 * ====================================================================== */
#[derive(Deserialize)]
struct ProofResponse {
    proof: String,
    #[serde(rename = "publicInputs")]
    public_inputs: Vec<String>,
}

fn generate_commitment() -> eyre::Result<(FixedBytes<32>, FixedBytes<32>, FixedBytes<32>)> {
    let root = repo_root();
    let script = root.join("scripts/js/generateCommitment.ts");

    let mut args: Vec<String> = vec![
        "tsx".to_string(),
        script.to_str().expect("valid script path").to_string(),
    ];
    let output = Command::new("npx").args(args).current_dir(&root).output()?;
    let s = String::from_utf8(output.stdout)?;
    let bytes = alloy::hex::decode(s)?;

    let mut a = [0u8; 32];
    a.copy_from_slice(&bytes[0..32]);
    let commitment = FixedBytes::<32>::from(a);
    let mut b = [0u8; 32];
    b.copy_from_slice(&bytes[32..64]);
    let nullifier = FixedBytes::<32>::from(b);
    let mut c = [0u8; 32];
    c.copy_from_slice(&bytes[64..96]);
    let secret = FixedBytes::<32>::from(c);

    Ok((commitment, nullifier, secret))
}

fn generate_proof(
    nullifier: FixedBytes<32>,
    secret: FixedBytes<32>,
    recipient: Address,
    leaves: Vec<FixedBytes<32>>,
) -> eyre::Result<(Vec<u8>, Vec<FixedBytes<32>>)> {
    let root = repo_root();
    let script = root.join("scripts/js/generateProof.ts");

    let mut args: Vec<String> = vec![
        "tsx".to_string(),
        script.to_str().expect("valid script path").to_string(),
        nullifier.to_string(),
        secret.to_string(),
        recipient.into_word().to_string(),
    ];
    for leaf in &leaves {
        args.push(leaf.to_string());
    }

    println!("GENERATE PROOF ARGS: {:?}\n", args);

    let output = Command::new("npx").args(args).current_dir(&root).output()?;
    let s = String::from_utf8(output.stdout)?;
    let resp: ProofResponse = serde_json::from_str(s.trim())?;
    let proof = hex_to_vec(&resp.proof)?;
    let public_inputs = resp
        .public_inputs
        .into_iter()
        .map(|s| hex_to_fixed_bytes(&s))
        .collect::<eyre::Result<Vec<FixedBytes<32>>>>()?;

    Ok((proof, public_inputs))
}

fn hex_to_vec(s: &str) -> eyre::Result<Vec<u8>> {
    let s = s.trim();
    Ok(alloy::hex::decode(s)?)
}

fn hex_to_fixed_bytes(s: &str) -> eyre::Result<FixedBytes<32>> {
    let bytes = hex_to_vec(s)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(FixedBytes::<32>::from(arr))
}

async fn deploy_imt(alice: &Account) -> Result<e2e::Receipt> {
    let imt_wasm = imt_wasm_path()?;
    let imt_rcpt = alice
        .as_deployer()
        .with_constructor(constructor!(uint!(15_U256)))
        .deploy_wasm(&imt_wasm)
        .await?;
    Ok(imt_rcpt)
}

async fn deploy_mixer(
    alice: &Account,
    verifier_addr: Address,
    imt_addr: Address,
) -> Result<e2e::Receipt> {
    let mixer_wasm = mixer_wasm_path()?;
    let mixer_rcpt = alice
        .as_deployer()
        .with_constructor(constructor!(verifier_addr, imt_addr))
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
    Ok(path)
}

fn imt_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "stylus_imt.wasm";
    let path = root
        .join("contracts/imt/target/wasm32-unknown-unknown/release")
        .join(file);
    Ok(path)
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

async fn block_timestamp(account: &Account) -> eyre::Result<u64> {
    let timestamp = account
        .wallet
        .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
        .await?
        .expect("latest block should exist")
        .header
        .timestamp;

    Ok(timestamp)
}

#[derive(Deserialize)]
struct ForgeCreateOutput {
    #[serde(rename = "deployedTo")]
    deployed_to: String,
}

fn deploy_verifier() -> eyre::Result<Address> {
    let root = repo_root();
    let mixer_dir = root.join("contracts/mixer");

    let status = Command::new("forge")
        .args(["build", "--sizes"])
        .current_dir(&mixer_dir)
        .status()
        .wrap_err("forge build failed")?;
    if !status.success() {
        return Err(eyre::eyre!(format!(
            "forge build exited with status {status}"
        )));
    }

    let output = Command::new("forge")
        .args([
            "create",
            "src/Verifier.sol:HonkVerifier",
            "--rpc-url",
            "http://localhost:8547",
            "--private-key",
            "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659",
            "--broadcast",
        ])
        .current_dir(&mixer_dir)
        .output()
        .wrap_err("forge create failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre::eyre!(format!(
            "forge create failed with status {status}: {stderr}",
            status = output.status
        )));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let deployed_to = if let Ok(parsed) = serde_json::from_str::<ForgeCreateOutput>(stdout.trim()) {
        parsed.deployed_to
    } else {
        stdout
            .lines()
            .find_map(|line| {
                line.split_once("Deployed to:")
                    .map(|(_, rhs)| rhs.trim().to_string())
            })
            .ok_or_else(|| eyre::eyre!("forge create output missing deployment address"))?
    };

    let address = Address::from_str(deployed_to.trim()).wrap_err("invalid verifier address")?;
    Ok(address)
}
