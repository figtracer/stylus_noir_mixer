#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

use ark_bn254::Fr;
use ark_ff::{BigInt, Field, PrimeField};
use stylus_sdk::alloy_primitives::U256;
use stylus_sdk::prelude::*;

const T: usize = 4;
const RATE: usize = 3;
const ROUNDS_F: usize = 8;
const ROUNDS_P: usize = 56;

pub struct Constants {
    internal_matrix_diagonal: [Fr; 4],
    round_constant: [[Fr; 4]; 64],
}

pub struct Sponge {
    iv: Fr,
    cache: [Fr; RATE],
    state: [Fr; T],
    cache_size: usize,
    squeeze_mode: bool,
    constants: Constants,
}

/* expose Poseidon2 as a deployable Stylus contract */
sol_storage! {
    #[entrypoint]
    pub struct Poseidon2 {}
}

#[public]
impl Poseidon2 {
    pub fn hash1(x: U256) -> U256 {
        hash_internal(&[x], 1, false)
    }
    pub fn hash2(x: U256, y: U256) -> U256 {
        hash_internal(&[x, y], 2, false)
    }
    pub fn hash3(x: U256, y: U256, z: U256) -> U256 {
        hash_internal(&[x, y, z], 3, false)
    }
}

fn fr_from_hex(hex: &str) -> Fr {
    let s = hex.strip_prefix("0x").unwrap_or(hex);
    let mut bytes = [0u8; 32];
    let mut bi = 31usize;
    let mut i = s.len();
    while i > 0 {
        let lo = s.as_bytes()[i - 1] as char;
        let lo = hex_val(lo);
        let hi = if i >= 2 {
            hex_val(s.as_bytes()[i - 2] as char)
        } else {
            0
        };
        bytes[bi] = (hi << 4) | lo;
        bi = bi.saturating_sub(1);
        i = i.saturating_sub(2);
        if i == 0 {
            break;
        }
    }
    Fr::from_be_bytes_mod_order(&bytes)
}

fn hex_val(c: char) -> u8 {
    match c {
        '0'..='9' => (c as u8) - b'0',
        'a'..='f' => (c as u8) - b'a' + 10,
        'A'..='F' => (c as u8) - b'A' + 10,
        _ => 0,
    }
}

fn load_constants() -> Constants {
    // Internal diagonal
    let internal_matrix_diagonal = [
        fr_from_hex("0x10dc6e9c006ea38b04b1e03b4bd9490c0d03f98929ca1d7fb56821fd19d3b6e7"),
        fr_from_hex("0x0c28145b6a44df3e0149b3d0a30b3bb599df9756d4dd9b84a86b38cfb45a740b"),
        fr_from_hex("0x00544b8338791518b2c7645a50392798b21f75bb60e3596170067d00141cac15"),
        fr_from_hex("0x222c01175718386f2e2e82eb122789e352e105a3b8fa852613bc534433ee428b"),
    ];

    // Round constants: start with zeros, then set the non-zero rows as provided.
    let mut round_constant = [[Fr::ZERO; 4]; 64];
    round_constant[0] = [
        fr_from_hex("0x19b849f69450b06848da1d39bd5e4a4302bb86744edc26238b0878e269ed23e5"),
        fr_from_hex("0x265ddfe127dd51bd7239347b758f0a1320eb2cc7450acc1dad47f80c8dcf34d6"),
        fr_from_hex("0x199750ec472f1809e0f66a545e1e51624108ac845015c2aa3dfc36bab497d8aa"),
        fr_from_hex("0x157ff3fe65ac7208110f06a5f74302b14d743ea25067f0ffd032f787c7f1cdf8"),
    ];
    round_constant[1] = [
        fr_from_hex("0x2e49c43c4569dd9c5fd35ac45fca33f10b15c590692f8beefe18f4896ac94902"),
        fr_from_hex("0x0e35fb89981890520d4aef2b6d6506c3cb2f0b6973c24fa82731345ffa2d1f1e"),
        fr_from_hex("0x251ad47cb15c4f1105f109ae5e944f1ba9d9e7806d667ffec6fe723002e0b996"),
        fr_from_hex("0x13da07dc64d428369873e97160234641f8beb56fdd05e5f3563fa39d9c22df4e"),
    ];
    round_constant[2] = [
        fr_from_hex("0x0c009b84e650e6d23dc00c7dccef7483a553939689d350cd46e7b89055fd4738"),
        fr_from_hex("0x011f16b1c63a854f01992e3956f42d8b04eb650c6d535eb0203dec74befdca06"),
        fr_from_hex("0x0ed69e5e383a688f209d9a561daa79612f3f78d0467ad45485df07093f367549"),
        fr_from_hex("0x04dba94a7b0ce9e221acad41472b6bbe3aec507f5eb3d33f463672264c9f789b"),
    ];
    round_constant[3] = [
        fr_from_hex("0x0a3f2637d840f3a16eb094271c9d237b6036757d4bb50bf7ce732ff1d4fa28e8"),
        fr_from_hex("0x259a666f129eea198f8a1c502fdb38fa39b1f075569564b6e54a485d1182323f"),
        fr_from_hex("0x28bf7459c9b2f4c6d8e7d06a4ee3a47f7745d4271038e5157a32fdf7ede0d6a1"),
        fr_from_hex("0x0a1ca941f057037526ea200f489be8d4c37c85bbcce6a2aeec91bd6941432447"),
    ];
    round_constant[4] = [
        fr_from_hex("0x0c6f8f958be0e93053d7fd4fc54512855535ed1539f051dcb43a26fd926361cf"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[5] = [
        fr_from_hex("0x123106a93cd17578d426e8128ac9d90aa9e8a00708e296e084dd57e69caaf811"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[6] = [
        fr_from_hex("0x26e1ba52ad9285d97dd3ab52f8e840085e8fa83ff1e8f1877b074867cd2dee75"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[7] = [
        fr_from_hex("0x1cb55cad7bd133de18a64c5c47b9c97cbe4d8b7bf9e095864471537e6a4ae2c5"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[8] = [
        fr_from_hex("0x1dcd73e46acd8f8e0e2c7ce04bde7f6d2a53043d5060a41c7143f08e6e9055d0"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[9] = [
        fr_from_hex("0x011003e32f6d9c66f5852f05474a4def0cda294a0eb4e9b9b12b9bb4512e5574"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[10] = [
        fr_from_hex("0x2b1e809ac1d10ab29ad5f20d03a57dfebadfe5903f58bafed7c508dd2287ae8c"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[11] = [
        fr_from_hex("0x2539de1785b735999fb4dac35ee17ed0ef995d05ab2fc5faeaa69ae87bcec0a5"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[12] = [
        fr_from_hex("0x0c246c5a2ef8ee0126497f222b3e0a0ef4e1c3d41c86d46e43982cb11d77951d"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[13] = [
        fr_from_hex("0x192089c4974f68e95408148f7c0632edbb09e6a6ad1a1c2f3f0305f5d03b527b"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[14] = [
        fr_from_hex("0x1eae0ad8ab68b2f06a0ee36eeb0d0c058529097d91096b756d8fdc2fb5a60d85"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[15] = [
        fr_from_hex("0x179190e5d0e22179e46f8282872abc88db6e2fdc0dee99e69768bd98c5d06bfb"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[16] = [
        fr_from_hex("0x29bb9e2c9076732576e9a81c7ac4b83214528f7db00f31bf6cafe794a9b3cd1c"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[17] = [
        fr_from_hex("0x225d394e42207599403efd0c2464a90d52652645882aac35b10e590e6e691e08"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[18] = [
        fr_from_hex("0x064760623c25c8cf753d238055b444532be13557451c087de09efd454b23fd59"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[19] = [
        fr_from_hex("0x10ba3a0e01df92e87f301c4b716d8a394d67f4bf42a75c10922910a78f6b5b87"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[20] = [
        fr_from_hex("0x0e070bf53f8451b24f9c6e96b0c2a801cb511bc0c242eb9d361b77693f21471c"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[21] = [
        fr_from_hex("0x1b94cd61b051b04dd39755ff93821a73ccd6cb11d2491d8aa7f921014de252fb"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[22] = [
        fr_from_hex("0x1d7cb39bafb8c744e148787a2e70230f9d4e917d5713bb050487b5aa7d74070b"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[23] = [
        fr_from_hex("0x2ec93189bd1ab4f69117d0fe980c80ff8785c2961829f701bb74ac1f303b17db"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[24] = [
        fr_from_hex("0x2db366bfdd36d277a692bb825b86275beac404a19ae07a9082ea46bd83517926"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[25] = [
        fr_from_hex("0x062100eb485db06269655cf186a68532985275428450359adc99cec6960711b8"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[26] = [
        fr_from_hex("0x0761d33c66614aaa570e7f1e8244ca1120243f92fa59e4f900c567bf41f5a59b"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[27] = [
        fr_from_hex("0x20fc411a114d13992c2705aa034e3f315d78608a0f7de4ccf7a72e494855ad0d"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[28] = [
        fr_from_hex("0x25b5c004a4bdfcb5add9ec4e9ab219ba102c67e8b3effb5fc3a30f317250bc5a"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[29] = [
        fr_from_hex("0x23b1822d278ed632a494e58f6df6f5ed038b186d8474155ad87e7dff62b37f4b"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[30] = [
        fr_from_hex("0x22734b4c5c3f9493606c4ba9012499bf0f14d13bfcfcccaa16102a29cc2f69e0"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[31] = [
        fr_from_hex("0x26c0c8fe09eb30b7e27a74dc33492347e5bdff409aa3610254413d3fad795ce5"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[32] = [
        fr_from_hex("0x070dd0ccb6bd7bbae88eac03fa1fbb26196be3083a809829bbd626df348ccad9"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[33] = [
        fr_from_hex("0x12b6595bdb329b6fb043ba78bb28c3bec2c0a6de46d8c5ad6067c4ebfd4250da"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[34] = [
        fr_from_hex("0x248d97d7f76283d63bec30e7a5876c11c06fca9b275c671c5e33d95bb7e8d729"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[35] = [
        fr_from_hex("0x1a306d439d463b0816fc6fd64cc939318b45eb759ddde4aa106d15d9bd9baaaa"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[36] = [
        fr_from_hex("0x28a8f8372e3c38daced7c00421cb4621f4f1b54ddc27821b0d62d3d6ec7c56cf"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[37] = [
        fr_from_hex("0x0094975717f9a8a8bb35152f24d43294071ce320c829f388bc852183e1e2ce7e"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[38] = [
        fr_from_hex("0x04d5ee4c3aa78f7d80fde60d716480d3593f74d4f653ae83f4103246db2e8d65"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[39] = [
        fr_from_hex("0x2a6cf5e9aa03d4336349ad6fb8ed2269c7bef54b8822cc76d08495c12efde187"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[40] = [
        fr_from_hex("0x2304d31eaab960ba9274da43e19ddeb7f792180808fd6e43baae48d7efcba3f3"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[41] = [
        fr_from_hex("0x03fd9ac865a4b2a6d5e7009785817249bff08a7e0726fcb4e1c11d39d199f0b0"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[42] = [
        fr_from_hex("0x00b7258ded52bbda2248404d55ee5044798afc3a209193073f7954d4d63b0b64"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[43] = [
        fr_from_hex("0x159f81ada0771799ec38fca2d4bf65ebb13d3a74f3298db36272c5ca65e92d9a"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[44] = [
        fr_from_hex("0x1ef90e67437fbc8550237a75bc28e3bb9000130ea25f0c5471e144cf4264431f"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[45] = [
        fr_from_hex("0x1e65f838515e5ff0196b49aa41a2d2568df739bc176b08ec95a79ed82932e30d"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[46] = [
        fr_from_hex("0x2b1b045def3a166cec6ce768d079ba74b18c844e570e1f826575c1068c94c33f"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[47] = [
        fr_from_hex("0x0832e5753ceb0ff6402543b1109229c165dc2d73bef715e3f1c6e07c168bb173"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[48] = [
        fr_from_hex("0x02f614e9cedfb3dc6b762ae0a37d41bab1b841c2e8b6451bc5a8e3c390b6ad16"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[49] = [
        fr_from_hex("0x0e2427d38bd46a60dd640b8e362cad967370ebb777bedff40f6a0be27e7ed705"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[50] = [
        fr_from_hex("0x0493630b7c670b6deb7c84d414e7ce79049f0ec098c3c7c50768bbe29214a53a"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[51] = [
        fr_from_hex("0x22ead100e8e482674decdab17066c5a26bb1515355d5461a3dc06cc85327cea9"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[52] = [
        fr_from_hex("0x25b3e56e655b42cdaae2626ed2554d48583f1ae35626d04de5084e0b6d2a6f16"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[53] = [
        fr_from_hex("0x1e32752ada8836ef5837a6cde8ff13dbb599c336349e4c584b4fdc0a0cf6f9d0"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[54] = [
        fr_from_hex("0x2fa2a871c15a387cc50f68f6f3c3455b23c00995f05078f672a9864074d412e5"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[55] = [
        fr_from_hex("0x2f569b8a9a4424c9278e1db7311e889f54ccbf10661bab7fcd18e7c7a7d83505"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[56] = [
        fr_from_hex("0x044cb455110a8fdd531ade530234c518a7df93f7332ffd2144165374b246b43d"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[57] = [
        fr_from_hex("0x227808de93906d5d420246157f2e42b191fe8c90adfe118178ddc723a5319025"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[58] = [
        fr_from_hex("0x02fcca2934e046bc623adead873579865d03781ae090ad4a8579d2e7a6800355"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[59] = [
        fr_from_hex("0x0ef915f0ac120b876abccceb344a1d36bad3f3c5ab91a8ddcbec2e060d8befac"),
        Fr::ZERO,
        Fr::ZERO,
        Fr::ZERO,
    ];
    round_constant[60] = [
        fr_from_hex("0x1797130f4b7a3e1777eb757bc6f287f6ab0fb85f6be63b09f3b16ef2b1405d38"),
        fr_from_hex("0x0a76225dc04170ae3306c85abab59e608c7f497c20156d4d36c668555decc6e5"),
        fr_from_hex("0x1fffb9ec1992d66ba1e77a7b93209af6f8fa76d48acb664796174b5326a31a5c"),
        fr_from_hex("0x25721c4fc15a3f2853b57c338fa538d85f8fbba6c6b9c6090611889b797b9c5f"),
    ];
    round_constant[61] = [
        fr_from_hex("0x0c817fd42d5f7a41215e3d07ba197216adb4c3790705da95eb63b982bfcaf75a"),
        fr_from_hex("0x13abe3f5239915d39f7e13c2c24970b6df8cf86ce00a22002bc15866e52b5a96"),
        fr_from_hex("0x2106feea546224ea12ef7f39987a46c85c1bc3dc29bdbd7a92cd60acb4d391ce"),
        fr_from_hex("0x21ca859468a746b6aaa79474a37dab49f1ca5a28c748bc7157e1b3345bb0f959"),
    ];
    round_constant[62] = [
        fr_from_hex("0x05ccd6255c1e6f0c5cf1f0df934194c62911d14d0321662a8f1a48999e34185b"),
        fr_from_hex("0x0f0e34a64b70a626e464d846674c4c8816c4fb267fe44fe6ea28678cb09490a4"),
        fr_from_hex("0x0558531a4e25470c6157794ca36d0e9647dbfcfe350d64838f5b1a8a2de0d4bf"),
        fr_from_hex("0x09d3dca9173ed2faceea125157683d18924cadad3f655a60b72f5864961f1455"),
    ];
    round_constant[63] = [
        fr_from_hex("0x0328cbd54e8c0913493f866ed03d218bf23f92d68aaec48617d4c722e5bd4335"),
        fr_from_hex("0x2bf07216e2aff0a223a487b1a7094e07e79e7bcc9798c648ee3347dd5329d34b"),
        fr_from_hex("0x1daf345a58006b736499c583cb76c316d6f78ed6a6dffc82111e11a63fe412df"),
        fr_from_hex("0x176563472456aaa746b694c60e1823611ef39039b2edc7ff391e6f2293d2c404"),
    ];

    Constants {
        internal_matrix_diagonal,
        round_constant,
    }
}

fn single_box(x: Fr) -> Fr {
    let s = x.square();
    s.square() * x
}

fn s_box(state: &mut [Fr; T]) {
    for i in 0..T {
        state[i] = single_box(state[i]);
    }
}

fn add_round_constants(state: &mut [Fr; T], rc: &[[Fr; 4]; 64], round: usize) {
    for i in 0..T {
        state[i] += rc[round][i];
    }
}

fn matrix_multiplication_4x4(input: &mut [Fr; 4]) {
    let t0 = input[0] + input[1];
    let t1 = input[2] + input[3];
    let mut t2 = input[1] + input[1];
    t2 += t1;
    let mut t3 = input[3] + input[3];
    t3 += t0;
    let mut t4 = t1 + t1;
    t4 += t4;
    t4 += t3;
    let mut t5 = t0 + t0;
    t5 += t5;
    t5 += t2;
    let t6 = t3 + t5;
    let t7 = t2 + t4;
    input[0] = t6;
    input[1] = t5;
    input[2] = t7;
    input[3] = t4;
}

fn internal_m_multiplication(input: &mut [Fr; 4], diag: &[Fr; 4]) {
    let mut sum = Fr::ZERO;
    for i in 0..4 {
        sum += input[i];
    }
    for i in 0..4 {
        input[i] = input[i] * diag[i] + sum;
    }
}

fn permutation(mut state: [Fr; 4], diag: &[Fr; 4], rc: &[[Fr; 4]; 64]) -> [Fr; 4] {
    matrix_multiplication_4x4(&mut state);

    let rf_first = ROUNDS_F / 2;
    for r in 0..rf_first {
        add_round_constants(&mut state, rc, r);
        s_box(&mut state);
        matrix_multiplication_4x4(&mut state);
    }

    let p_end = rf_first + ROUNDS_P;
    for r in rf_first..p_end {
        state[0] += rc[r][0];
        state[0] = single_box(state[0]);
        internal_m_multiplication(&mut state, diag);
    }

    let num_rounds = ROUNDS_F + ROUNDS_P;
    for r in p_end..num_rounds {
        add_round_constants(&mut state, rc, r);
        s_box(&mut state);
        matrix_multiplication_4x4(&mut state);
    }

    state
}

fn new_poseidon2(iv: Fr, constants: Constants) -> Sponge {
    let mut s = Sponge {
        iv,
        cache: [Fr::ZERO; RATE],
        state: [Fr::ZERO; T],
        cache_size: 0,
        squeeze_mode: false,
        constants,
    };
    s.state[RATE] = iv;
    s
}

fn perform_duplex(sp: &mut Sponge) -> [Fr; RATE] {
    for i in 0..RATE {
        if i >= sp.cache_size {
            sp.cache[i] = Fr::ZERO;
        }
    }
    for i in 0..RATE {
        sp.state[i] += sp.cache[i];
    }
    sp.state = permutation(
        sp.state,
        &sp.constants.internal_matrix_diagonal,
        &sp.constants.round_constant,
    );
    let mut out = [Fr::ZERO; RATE];
    for i in 0..RATE {
        out[i] = sp.state[i];
    }
    out
}

fn absorb(sp: &mut Sponge, input: Fr) {
    if !sp.squeeze_mode && sp.cache_size == RATE {
        let _ = perform_duplex(sp);
        sp.cache[0] = input;
        sp.cache_size = 1;
    } else if !sp.squeeze_mode && sp.cache_size != RATE {
        sp.cache[sp.cache_size] = input;
        sp.cache_size += 1;
    }
}

fn squeeze(sp: &mut Sponge) -> Fr {
    if !sp.squeeze_mode {
        let new_out = perform_duplex(sp);
        sp.squeeze_mode = true;
        for i in 0..RATE {
            sp.cache[i] = new_out[i];
        }
        sp.cache_size = RATE;
    }

    let result = sp.cache[0];
    for i in 1..RATE {
        if i < sp.cache_size {
            sp.cache[i - 1] = sp.cache[i];
        }
    }
    sp.cache_size -= 1;
    sp.cache[sp.cache_size] = Fr::ZERO;
    result
}

fn iv_from_len(len: usize) -> Fr {
    let iv_u256 = U256::from(len as u128) << 64;
    fr_from_u256(iv_u256)
}

fn fr_from_u256(x: U256) -> Fr {
    let bytes = x.to_be_bytes::<32>();
    Fr::from_be_bytes_mod_order(&bytes)
}

fn u256_from_fr(x: Fr) -> U256 {
    let bi: BigInt<4> = x.into_bigint(); /* little-endian limbs: [l0, l1, l2, l3] */
    let limbs = bi.0;
    let mut bytes = [0u8; 32];
    for (i, limb) in limbs.iter().rev().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_be_bytes());
    }
    U256::from_be_bytes(bytes)
}

fn hash_internal(inputs: &[U256], std_input_length: usize, is_variable_length: bool) -> U256 {
    let constants = load_constants();
    let mut sp = new_poseidon2(iv_from_len(inputs.len()), constants);
    for (i, x) in inputs.iter().enumerate() {
        if i < std_input_length {
            absorb(&mut sp, fr_from_u256(*x));
        }
    }
    if is_variable_length {
        absorb(&mut sp, Fr::from(1u64));
    }
    u256_from_fr(squeeze(&mut sp))
}
