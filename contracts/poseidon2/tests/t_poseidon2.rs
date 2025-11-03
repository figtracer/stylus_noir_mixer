#[cfg(test)]
use stylus_poseidon2::Poseidon2;

use alloy_primitives::U256;
use hex as _;

fn u256(hex: &str) -> U256 {
    let mut h = hex.strip_prefix("0x").unwrap_or(hex).to_string();
    if h.len() % 2 == 1 {
        h.insert(0, '0');
    }
    let bytes = hex::decode(h).expect("invalid hex");
    assert!(bytes.len() <= 32, "hex too wide for U256");
    let mut be = [0u8; 32];
    let start = 32 - bytes.len();
    be[start..].copy_from_slice(&bytes);
    U256::from_be_bytes(be)
}

/* credits: https://github.com/zemse/poseidon2-evm/blob/main/src/Poseidon2Lib.sol
*  the hashed values are taken from the solidity implementation of Poseidon2
*/

#[test]
fn matches_solidity_poseidon2_hash2() {
    let vectors: &[(&str, &str, &str)] = &[
        (
            "0x0",
            "0x0",
            "0x0b63a53787021a4a962a452c2921b3663aff1ffd8d5510540f8e659e782956f1",
        ),
        (
            "0x1",
            "0x2",
            "0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383",
        ),
        (
            "0x7b",
            "0x1c8",
            "0x148c4666e2c5bce33dc53b30430808464f7ed76b28daf8a36722e93b49a31a5e",
        ),
        (
            "0x2a",
            "0x0",
            "0x1e7981f857394b926eee0742ad9104aa612ffe6873522a4eccaab935b2ff3c96",
        ),
        (
            "0x0",
            "0x2a",
            "0x1b1fd004aab577fc6cd7c3bb807079273b0ba27c6b0a168e5a8106dafbf8a249",
        ),
        (
            "0x7f3b7",
            "0x5f4b57b21c75007b",
            "0x10953cfc12d9e84c908ad271373689e4146d80e103e9ab429fdb883e03c297d4",
        ),
        (
            "0x6d026db998e839b5967c2",
            "0x41d073d77ab9c4d910944",
            "0x1c3360753c23bff6705c50bdb90284eac323ff2b4985483213b14eb755740d64",
        ),
        (
            "0x2fd1c265352ef9",
            "0x157d9dd78937101",
            "0x1025ecca5d719d1bb74274700e33f532b231bd08bbdba184da61a0101c15933a",
        ),
    ];

    for (x_hex, y_hex, h_hex) in vectors {
        let x = u256(x_hex);
        let y = u256(y_hex);
        let got = Poseidon2::hash2(x, y);
        let expected = u256(h_hex);
        println!(
            "x={} y={} -> got={} expected={}",
            x_hex,
            y_hex,
            format!("{:#066x}", got),
            format!("{:#066x}", expected)
        );
        assert_eq!(
            format!("{:#066x}", got),
            format!("{:#066x}", expected),
            "mismatch for x={}, y={}",
            x_hex,
            y_hex
        );
    }
}
