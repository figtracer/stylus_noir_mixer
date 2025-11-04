use eyre::Result;
use std::process::Command;

#[test]
fn test_get_commitment() -> Result<()> {
    let output = Command::new("npx")
        .arg("tsx")
        .arg("../js-scripts/generateCommitment.ts")
        .output()?;

    assert!(
        output.status.success(),
        "generateCommitment.ts failed to run"
    );

    let stdout = String::from_utf8(output.stdout)?;
    let hex = stdout.trim();
    let hex = hex.strip_prefix("0x").unwrap_or(hex);

    assert_eq!(hex.len(), 32 * 3 * 2, "unexpected encoded length");

    let raw = hex::decode(hex)?;
    assert_eq!(raw.len(), 96);

    let commitment = &raw[0..32];
    let nullifier = &raw[32..64];
    let secret = &raw[64..96];

    fn non_zero(slice: &[u8]) -> bool {
        slice.iter().any(|b| *b != 0)
    }
    assert!(non_zero(commitment), "commitment should be non-zero");
    assert!(non_zero(nullifier), "nullifier should be non-zero");
    assert!(non_zero(secret), "secret should be non-zero");

    Ok(())
}
