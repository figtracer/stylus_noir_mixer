#![cfg(feature = "e2e")]

use alloy::{eips::BlockId, providers::Provider, rpc::types::BlockTransactionsKind};
use alloy_primitives::{uint, Address, Bytes as AlloyBytes, FixedBytes, U256};
use alloy_sol_types::SolValue;
use e2e::{constructor, receipt, send, Account, Revert};
use eyre::Result;
use std::path::PathBuf;
use std::process::Command;

mod abi;
use abi::MixerAbi;

const DENOMINATION: U256 = uint!(1_000_000_000_000_000_000_U256);

/* ======================================================================
 *                      generate commmitment and proof
 * ====================================================================== */
#[e2e::test]
async fn generate_commitment_and_proof_works(alice: Account) -> Result<()> {
    let (commitment, nullifier, secret) = generate_commitment()?;
    let leaves = vec![commitment];
    let recipient = alice.address();
    println!(
        "commitment: 0x{}\nnullifier: 0x{}\nsecret: 0x{}",
        alloy::hex::encode(commitment.as_slice()),
        alloy::hex::encode(nullifier.as_slice()),
        alloy::hex::encode(secret.as_slice()),
    );

    let (proof, public_inputs) = generate_proof(nullifier, secret, recipient, leaves)?;
    println!("public_inputs: {:?}", public_inputs);
    println!("proof: 0x{}", alloy::hex::encode(proof.as_slice()));
    Ok(())
}
/* ======================================================================
 *                               deposit()
 * ====================================================================== */
// #[e2e::test]
// async fn mixer_deposit_works(alice: Account) -> Result<()> {
//     let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
//     let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
//     let verifier_addr = Address::ZERO; /* not needed for deposit path */
//     let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
//         .await?
//         .contract_address;
//     let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

//     /* generate commitment */
//     let (commitment, _nullifier, _secret) = generate_commitment()?;

//     let rcpt = receipt!(mixer.deposit(commitment).value(DENOMINATION))?;

//     /* record timestamp right after the deposit */
//     let timestamp = U256::from(block_timestamp(&alice).await?);

//     let raw_log = rcpt.inner.as_receipt().unwrap().logs.first().unwrap();
//     let decoded = raw_log
//         .log_decode::<MixerAbi::Deposit>()
//         .expect("decode deposit event");

//     let event = &decoded.inner.data;
//     assert_eq!(event.commitment, commitment);
//     assert_eq!(event.timestamp, timestamp);
//     assert_eq!(event.index, 1u32);
//     Ok(())
// }

// #[e2e::test]
// async fn mixer_deposit_rejects_invalid_denomination(alice: Account) -> Result<()> {
//     let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
//     let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
//     let verifier_addr = Address::ZERO; /* not needed for deposit path */
//     let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
//         .await?
//         .contract_address;
//     let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

//     /* generate commitment */
//     let (commitment, _nullifier, _secret) = generate_commitment()?;

//     /* call deposit with zero value -> expect revert */
//     let err = send!(mixer.deposit(commitment).value(U256::ZERO)).expect_err("should revert");
//     assert!(err.reverted_with(MixerAbi::InvalidDenomination {}));
//     Ok(())
// }

// #[e2e::test]
// async fn mixer_deposit_rejects_duplicate_commitment(alice: Account) -> Result<()> {
//     let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
//     let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
//     let verifier_addr = Address::ZERO; /* not needed for deposit path */
//     let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
//         .await?
//         .contract_address;
//     let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

//     /* generate commitment */
//     let (commitment, _nullifier, _secret) = generate_commitment()?;

//     receipt!(mixer.deposit(commitment).value(DENOMINATION))?;
//     let err = send!(mixer.deposit(commitment).value(DENOMINATION)).expect_err("should revert");
//     assert!(err.reverted_with(MixerAbi::CommitmentAlreadyExists {}));
//     Ok(())
// }

/* ======================================================================
 *                               withdraw()
 * ====================================================================== */
// #[e2e::test]
// async fn mixer_withdraw_works(alice: Account) -> Result<()> {
//     /* deploy poseidon */
//     let poseidon_addr = deploy_poseidon(&alice).await?.contract_address;
//     /* deploy imt */
//     let imt_addr = deploy_imt(&alice, poseidon_addr).await?.contract_address;
//     /* deploy verifier */
//     let verifier_addr = Address::ZERO; /* not needed for deposit path */
//     /* deploy mixer */
//     let mixer_addr = deploy_mixer(&alice, verifier_addr, poseidon_addr, imt_addr)
//         .await?
//         .contract_address;
//     let mixer = MixerAbi::new(mixer_addr, &alice.wallet);

//     /* generate commitment */
//     let (commitment, _nullifier, _secret) = generate_commitment()?;

//     receipt!(mixer.deposit(commitment).value(DENOMINATION))?;

//     /* generate proof */
//     let proof = generate_proof()?;
//     println!("proof: 0x{}", alloy::hex::encode(proof.as_slice()),);

//     receipt!(mixer
//         .withdraw(proof, commitment, nullifier_hash, alice.address)
//         .value(DENOMINATION))?;
//     Ok(())
// }
/* ======================================================================
 *                               INTERNAL HELPERS
 * ====================================================================== */
fn generate_commitment() -> eyre::Result<(FixedBytes<32>, FixedBytes<32>, FixedBytes<32>)> {
    let root = repo_root();
    let script = root.join("scripts/js/generateCommitment.ts");
    let output = Command::new("npx")
        .args(["tsx", script.to_str().expect("valid script path")])
        .current_dir(&root)
        .output()?;

    let s = String::from_utf8(output.stdout)?;
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(&s);
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

    let recipient_word: FixedBytes<32> = recipient.into_word();
    let mut args: Vec<String> = vec![
        "tsx".to_string(),
        script.to_str().expect("valid script path").to_string(),
        nullifier.to_string(),
        secret.to_string(),
        recipient_word.to_string(),
    ];
    for leaf in &leaves {
        args.push(leaf.to_string());
    }
    println!("args: {:?}", args);
    let output = Command::new("npx").args(args).current_dir(&root).output()?;

    let s = String::from_utf8(output.stdout)?;
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(&s);
    let bytes = alloy::hex::decode(s)?;

    /* The returned hex string is encoded via `AbiCoder#encode(["bytes", "bytes32[]"], ...)`, which
     * is a top-level encoding of two parameters, not a nested tuple. Therefore we must decode it
     * with `inside_tuple = false` (see `SolValue::abi_decode` docs).
     */
    let (proof_bytes, public_inputs): (AlloyBytes, Vec<FixedBytes<32>>) =
        <(AlloyBytes, Vec<FixedBytes<32>>)>::abi_decode(&bytes, false)?;

    Ok((proof_bytes.to_vec(), public_inputs))
}

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
    Ok(path)
}

/* get WASM paths */
fn poseidon_wasm_path() -> eyre::Result<PathBuf> {
    let root = repo_root();
    let file = "openzeppelin_poseidon.wasm";
    let path = root
        .join("contracts/poseidon/target/wasm32-unknown-unknown/release")
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
