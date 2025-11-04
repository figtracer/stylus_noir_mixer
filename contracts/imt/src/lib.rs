#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

use stylus_common::errors::ContractErrors;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256, U32},
    evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageArray, StorageFixedBytes, StorageMap, StorageU32},
    stylus_core::calls::context::Call,
};

const ROOT_HISTORY_SIZE_U32: u32 = 30;

#[storage]
pub struct IMTContract {
    depth: StorageU32,
    current_root_index: StorageU32,
    next_leaf_index: StorageU32,
    cached_subtrees: StorageMap<U32, StorageFixedBytes<32>>,
    roots: StorageArray<StorageFixedBytes<32>, 30>,
    hasher: StorageAddress,
}

/* exposed methods */
#[public]
impl IMTContract {
    #[constructor]
    pub fn initialize(&mut self, depth: U32) -> Result<(), ContractErrors> {
        let d: u32 = u32::from_be_bytes(depth.to_be_bytes::<4>());
        if d == 0 || d >= 32 {
            return Err(ContractErrors::invalid_depth());
        }
        self.depth.set(depth);
        self.current_root_index.set(U32::from(0u32));
        self.next_leaf_index.set(U32::from(0u32));
        /* initialize the tree with the zero hashes */
        let init_root = zeros_u32(d - 1);
        self.roots.setter(U32::from(0u32)).unwrap().set(init_root);
        Ok(())
    }

    /* todo: this should only be called by the owner of the contract */
    pub fn set_hasher(&mut self, addr: Address) -> Result<(), ContractErrors> {
        self.hasher.set(addr);
        Ok(())
    }

    pub fn insert(&mut self, leaf: FixedBytes<32>) -> Result<U32, ContractErrors> {
        let depth_u32: u32 = u32::from_be_bytes(self.depth.get().to_be_bytes::<4>());
        if depth_u32 == 0 || depth_u32 >= 32 {
            return Err(ContractErrors::invalid_depth());
        }

        let next_idx_u32: u32 = u32::from_be_bytes(self.next_leaf_index.get().to_be_bytes::<4>());
        let capacity: u64 = 1u64 << depth_u32;
        if (next_idx_u32 as u64) == capacity {
            return Err(ContractErrors::tree_is_full());
        }

        let mut current_index: u32 = next_idx_u32;
        let mut current_hash: FixedBytes<32> = leaf;
        let mut left: FixedBytes<32>;
        let mut right: FixedBytes<32>;

        for i in 0..depth_u32 {
            if current_index % 2 == 0 {
                left = current_hash;
                right = zeros_u32(i as u32);
                self.cached_subtrees
                    .insert(U32::from(i as u32), current_hash);
            } else {
                let guard = self.cached_subtrees.getter(U32::from(i as u32));
                left = guard.get();
                right = current_hash;
            }

            current_hash = self.hash_pair(left, right);
            current_index /= 2;
        }

        let cur_root_idx: u32 =
            u32::from_be_bytes(self.current_root_index.get().to_be_bytes::<4>());
        let new_root_idx = (cur_root_idx + 1) % ROOT_HISTORY_SIZE_U32;
        self.current_root_index.set(U32::from(new_root_idx));
        self.roots
            .setter(U32::from(new_root_idx))
            .unwrap()
            .set(current_hash);

        self.next_leaf_index.set(U32::from(next_idx_u32 + 1));
        Ok(U32::from(next_idx_u32))
    }

    pub fn is_known_root(&self, root: FixedBytes<32>) -> bool {
        if root == FixedBytes::<32>::ZERO {
            return false;
        }

        let current_root_index_u32: u32 =
            u32::from_be_bytes(self.current_root_index.get().to_be_bytes::<4>());
        let mut i = current_root_index_u32;
        loop {
            let guard = self.roots.getter(U32::from(i)).unwrap();
            if guard.get() == root {
                return true;
            }
            if i == 0 {
                i = ROOT_HISTORY_SIZE_U32;
            }
            i -= 1;
            if i == current_root_index_u32 {
                break;
            }
        }
        false
    }

    /* public pure equivalent */
    pub fn zeros(i: U256) -> FixedBytes<32> {
        let i_u32: u32 = {
            let bytes = i.to_be_bytes::<32>();
            u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])
        };
        zeros_u32(i_u32)
    }
}

/* internal helpers */

fn hash_pair(
    hasher_address: Address,
    left: FixedBytes<32>,
    right: FixedBytes<32>,
) -> FixedBytes<32> {
    let hasher = IHasher::new(hasher_address);
    let config = Call::new().gas(evm::gas_left() / 2).value(msg::value());

    let out = hasher
        .hash_2(config, u256_from_b32(left), u256_from_b32(right))
        .expect("hash_2 call failed");
    b32_from_u256(out)
}
fn zeros_u32(i: u32) -> FixedBytes<32> {
    match i {
        0 => fb32("0x0d823319708ab99ec915efd4f7e03d11ca1790918e8f04cd14100aceca2aa9ff"),
        1 => fb32("0x170a9598425eb05eb8dc06986c6afc717811e874326a79576c02d338bdf14f13"),
        2 => fb32("0x273b1a40397b618dac2fc66ceb71399a3e1a60341e546e053cbfa5995e824caf"),
        3 => fb32("0x16bf9b1fb2dfa9d88cfb1752d6937a1594d257c2053dff3cb971016bfcffe2a1"),
        4 => fb32("0x1288271e1f93a29fa6e748b7468a77a9b8fc3db6b216ce5fc2601fc3e9bd6b36"),
        5 => fb32("0x1d47548adec1068354d163be4ffa348ca89f079b039c9191378584abd79edeca"),
        6 => fb32("0x0b98a89e6827ef697b8fb2e280a2342d61db1eb5efc229f5f4a77fb333b80bef"),
        7 => fb32("0x231555e37e6b206f43fdcd4d660c47442d76aab1ef552aef6db45f3f9cf2e955"),
        8 => fb32("0x03d0dc8c92e2844abcc5fdefe8cb67d93034de0862943990b09c6b8e3fa27a86"),
        9 => fb32("0x1d51ac275f47f10e592b8e690fd3b28a76106893ac3e60cd7b2a3a443f4e8355"),
        10 => fb32("0x16b671eb844a8e4e463e820e26560357edee4ecfdbf5d7b0a28799911505088d"),
        11 => fb32("0x115ea0c2f132c5914d5bb737af6eed04115a3896f0d65e12e761ca560083da15"),
        12 => fb32("0x139a5b42099806c76efb52da0ec1dde06a836bf6f87ef7ab4bac7d00637e28f0"),
        13 => fb32("0x0804853482335a6533eb6a4ddfc215a08026db413d247a7695e807e38debea8e"),
        14 => fb32("0x2f0b264ab5f5630b591af93d93ec2dfed28eef017b251e40905cdf7983689803"),
        15 => fb32("0x170fc161bf1b9610bf196c173bdae82c4adfd93888dc317f5010822a3ba9ebee"),
        16 => fb32("0x0b2e7665b17622cc0243b6fa35110aa7dd0ee3cc9409650172aa786ca5971439"),
        17 => fb32("0x12d5a033cbeff854c5ba0c5628ac4628104be6ab370699a1b2b4209e518b0ac5"),
        18 => fb32("0x1bc59846eb7eafafc85ba9a99a89562763735322e4255b7c1788a8fe8b90bf5d"),
        19 => fb32("0x1b9421fbd79f6972a348a3dd4721781ec25a5d8d27342942ae00aba80a3904d4"),
        20 => fb32("0x087fde1c4c9c27c347f347083139eee8759179d255ec8381c02298d3d6ccd233"),
        21 => fb32("0x1e26b1884cb500b5e6bbfdeedbdca34b961caf3fa9839ea794bfc7f87d10b3f1"),
        22 => fb32("0x09fc1a538b88bda55a53253c62c153e67e8289729afd9b8bfd3f46f5eecd5a72"),
        23 => fb32("0x14cd0edec3423652211db5210475a230ca4771cd1e45315bcd6ea640f14077e2"),
        24 => fb32("0x1d776a76bc76f4305ef0b0b27a58a9565864fe1b9f2a198e8247b3e599e036ca"),
        25 => fb32("0x1f93e3103fed2d3bd056c3ac49b4a0728578be33595959788fa25514cdb5d42f"),
        26 => fb32("0x138b0576ee7346fb3f6cfb632f92ae206395824b9333a183c15470404c977a3b"),
        27 => fb32("0x0745de8522abfcd24bd50875865592f73a190070b4cb3d8976e3dbff8fdb7f3d"),
        28 => fb32("0x2ffb8c798b9dd2645e9187858cb92a86c86dcd1138f5d610c33df2696f5f6860"),
        29 => fb32("0x2612a1395168260c9999287df0e3c3f1b0d8e008e90cd15941e4c2df08a68a5a"),
        30 => fb32("0x10ebedce66a910039c8edb2cd832d6a9857648ccff5e99b5d08009b44b088edf"),
        31 => fb32("0x213fb841f9de06958cf4403477bdbff7c59d6249daabfee147f853db7c808082"),
        _ => panic!("index out of bounds"),
    }
}

/* FixedBytes<32> to u256 */
fn u256_from_b32(b: FixedBytes<32>) -> U256 {
    let mut arr = [0u8; 32];
    arr.copy_from_slice(b.as_slice());
    U256::from_be_bytes(arr)
}

/* u256 to FixedBytes<32> */
fn b32_from_u256(x: U256) -> FixedBytes<32> {
    FixedBytes::<32>::from(x.to_be_bytes::<32>())
}

/* hex literal to FixedBytes<32> */
fn fb32(hex_str: &str) -> FixedBytes<32> {
    let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let mut bytes = [0u8; 32];
    let mut bi = 31usize;
    let mut i = s.len();
    while i > 0 {
        let lo = hex_val(s.as_bytes()[i - 1] as char);
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
    FixedBytes::<32>::from(bytes)
}

/* hex value to u8 */
fn hex_val(c: char) -> u8 {
    match c {
        '0'..='9' => (c as u8) - b'0',
        'a'..='f' => (c as u8) - b'a' + 10,
        'A'..='F' => (c as u8) - b'A' + 10,
        _ => 0,
    }
}
