#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

pub mod interface;

use openzeppelin_poseidon::interface::PoseidonInterface as IPoseidon;

use stylus_common::errors::ContractErrors;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256, U32},
    call::Call,
    prelude::*,
    storage::{StorageAddress, StorageArray, StorageFixedBytes, StorageMap, StorageU32},
};

const ROOT_HISTORY_SIZE_U32: u32 = 30;

#[cfg(feature = "contract")]
#[entrypoint]
#[storage]
pub struct IMT {
    depth: StorageU32,
    current_root_index: StorageU32,
    next_leaf_index: StorageU32,
    cached_subtrees: StorageMap<U32, StorageFixedBytes<32>>,
    roots: StorageArray<StorageFixedBytes<32>, 30>,
    hasher: StorageAddress,
}

/* ======================================================================
 *                               Contract
 * ====================================================================== */
#[cfg(feature = "contract")]
#[public]
impl IMT {
    #[constructor]
    fn initialize(&mut self, depth: U32, hasher: Address) -> Result<(), ContractErrors> {
        let d: u32 = u32::from_be_bytes(depth.to_be_bytes::<4>());
        if d == 0 || d >= 32 {
            return Err(ContractErrors::invalid_depth());
        }
        self.depth.set(depth);
        self.current_root_index.set(U32::from(0u32));
        self.next_leaf_index.set(U32::from(0u32));
        let init_root = Self::zeros_u32(d);
        self.roots.setter(U32::from(0u32)).unwrap().set(init_root);
        self.hasher.set(hasher);
        Ok(())
    }

    fn insert(&mut self, leaf: FixedBytes<32>) -> Result<U32, ContractErrors> {
        let depth_u32: u32 = u32::from_be_bytes(self.depth.get().to_be_bytes::<4>());
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
            if (current_index & 1) == 0 {
                left = current_hash;
                right = Self::zeros_u32(i);
                self.cached_subtrees.setter(U32::from(i)).set(current_hash);
            } else {
                left = self.cached_subtrees.getter(U32::from(i)).get();
                right = current_hash;
            }

            current_hash = self.hash_pair(left, right);
            current_index >>= 1;
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
        Ok(self.next_leaf_index.get())
    }

    fn is_known_root(&self, root: FixedBytes<32>) -> bool {
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

    /* getters */
    fn get_hasher(&self) -> Address {
        self.hasher.get()
    }

    fn get_depth(&self) -> U32 {
        self.depth.get()
    }

    fn get_current_root_index(&self) -> U32 {
        self.current_root_index.get()
    }

    fn get_next_leaf_index(&self) -> U32 {
        self.next_leaf_index.get()
    }

    fn zeros(&self, i: U256) -> FixedBytes<32> {
        let i_u32: u32 = {
            let bytes = i.to_be_bytes::<32>();
            u32::from_be_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])
        };
        Self::zeros_u32(i_u32)
    }
}

/* ======================================================================
 *                               INTERNAL HELPERS
 * ====================================================================== */
#[cfg(feature = "contract")]
impl IMT {
    fn hash_pair(&mut self, left: FixedBytes<32>, right: FixedBytes<32>) -> FixedBytes<32> {
        let hasher = IPoseidon::new(self.hasher.get());
        let out = hasher
            .hash(
                Call::new(),
                [Self::u256_from_b32(left), Self::u256_from_b32(right)],
            )
            .expect("poseidon hash call failed");
        Self::b32_from_u256(out)
    }

    fn zeros_u32(i: u32) -> FixedBytes<32> {
        match i {
            0 => Self::fb32("0x168db4aa1d4e4bf2ee46eb882e1c38a7de1a4da47e17b207a5494a14605ae38e"),
            1 => Self::fb32("0x257a568bdc9cc663b2cf123f7d7b6c5eedd5a312d2792305352e09f1733a56b5"),
            2 => Self::fb32("0x25b9b4ff326c7783ce7a3ae1503dce4552211bdfb510808e215f4227da087023"),
            3 => Self::fb32("0x0aa6931cdcc4482ac0a053cf28a380154ce6500cc02087ea9c8b71ffe597ea59"),
            4 => Self::fb32("0x20cb91532baf018f130fc336438e923c9f2de935efdd4325a8c7eda10d5c5520"),
            5 => Self::fb32("0x1ca38bd416b196d58f59133a826b64ec9f697e854ea8f10b9337c74365e79068"),
            6 => Self::fb32("0x1d09e36bc1db6b3e83298d8045cda770ca55eaeff1da0d44e684647653a1a185"),
            7 => Self::fb32("0x266afaeab47b775c2275cde3248b68503f3079eca6461c1907fec9b979afe9ff"),
            8 => Self::fb32("0x22794d6b26dd7398aa4f3c7d58ed5ea48f698ff4b229d21442846d8cd70959b1"),
            9 => Self::fb32("0x05e208e2e76bcfe61cb39a79c0e263ee7874ba71cd64bc54e8bafd470055c6ef"),
            10 => Self::fb32("0x26c093f627ffb8a25ab933cf64dd4f29dae2b103b48db3bf619f0dc39b298222"),
            11 => Self::fb32("0x058676dab63180e26827fc2d2feccd6b191aa0e6589aa589398addb28e71a011"),
            12 => Self::fb32("0x0f9ba00d2e0001bed485a0a1c2416e1aa2c86bf7c859c6707d0169170678f174"),
            13 => Self::fb32("0x06fa06667c34201bcd5f6334de6b8c0b22b5f6bc57e401ed7660c40afd880b26"),
            14 => Self::fb32("0x26ec3289eb146620b56807d58b3fae45adb7d7dfdc0a65194333e6dc2aa3de9e"),
            15 => Self::fb32("0x2d2f60a05d456896411242de0eff23497c889f762e2eb5db0a07df329f452a92"),
            16 => Self::fb32("0x1ee903a4eac57310c624c0e30f2bd083eb68a595306df83b1111db0fffce45ea"),
            17 => Self::fb32("0x05f96e491710c7e1d65207b36e0031c1de403eb32753de2489e8abce4c2e86ff"),
            18 => Self::fb32("0x2375b170da8f212cf2b23538990cb6a2e319c50eee555a3fcbed25946326be6c"),
            19 => Self::fb32("0x14307dca3f2b6224ff19c5c0a19129c5fa79d48c645ebb1c5302cb41a131e72a"),
            20 => Self::fb32("0x051e91aeea86b05dcd2b5218126fb3cf3990c81d53f0947028a933026eb94b3a"),
            21 => Self::fb32("0x089fcba3da069d909de7d9ea88e8c7bb49d5934947e53a6a9c1f5eb662c27f2e"),
            22 => Self::fb32("0x1f36ef937da9689c0f70364036741031e0f2b87eef22c735019371d4396c6b3f"),
            23 => Self::fb32("0x21ad2f97aebd9fcd4c41433e5db8d4f64c863360ddecc343cd500b62180134ee"),
            24 => Self::fb32("0x0acf88c5a38c8f9279a313844e7af8026eeeaceeabad6b582d9f3fb123b62d70"),
            25 => Self::fb32("0x1175dd405c3e38670903ef572cfa3ccf441a7cf70c88f8fd8be1b46f33915970"),
            26 => Self::fb32("0x1881fadef7baed7dab5ec3999ade81286cd070fdf234fc1b451e34a3bc4231c6"),
            27 => Self::fb32("0x2219be343fd079048377a2560467d13407dd0e99dc810aa82aa014c617dd1b9e"),
            28 => Self::fb32("0x20311205434cb48873977f5a36fc509939eae9e64cf66395df8ea03264c7798b"),
            29 => Self::fb32("0x29084b176141114483fc6c6eb4444a01c644361a2c004a7efd2e252d8cce4e70"),
            30 => Self::fb32("0x0bad0880fc04d5a9b2fccef498448214d50b92a973390a93ca7533ff02fc7721"),
            31 => Self::fb32("0x13b6403089d691e83af7392d8e9bddd76e83d8204b2475fc6c60679bd338dea8"),
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
            let lo = Self::hex_val(s.as_bytes()[i - 1] as char);
            let hi = if i >= 2 {
                Self::hex_val(s.as_bytes()[i - 2] as char)
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
}
