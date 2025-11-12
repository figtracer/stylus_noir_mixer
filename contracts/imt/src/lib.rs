#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

pub mod interface;

use openzeppelin_poseidon::hash_two_fixed_bytes;

use stylus_common::errors::ContractErrors;
use stylus_sdk::{
    alloy_primitives::{fixed_bytes, FixedBytes, U256, U32},
    prelude::*,
    storage::{StorageArray, StorageFixedBytes, StorageMap, StorageU32},
};

/* constructor args */
const ROOT_HISTORY_SIZE_U32: u32 = 30;

#[cfg(feature = "contract")]
const ZERO_LEAVES: [FixedBytes<32>; 16] = [
    fixed_bytes!("0x168db4aa1d4e4bf2ee46eb882e1c38a7de1a4da47e17b207a5494a14605ae38e"),
    fixed_bytes!("0x257a568bdc9cc663b2cf123f7d7b6c5eedd5a312d2792305352e09f1733a56b5"),
    fixed_bytes!("0x25b9b4ff326c7783ce7a3ae1503dce4552211bdfb510808e215f4227da087023"),
    fixed_bytes!("0x0aa6931cdcc4482ac0a053cf28a380154ce6500cc02087ea9c8b71ffe597ea59"),
    fixed_bytes!("0x20cb91532baf018f130fc336438e923c9f2de935efdd4325a8c7eda10d5c5520"),
    fixed_bytes!("0x1ca38bd416b196d58f59133a826b64ec9f697e854ea8f10b9337c74365e79068"),
    fixed_bytes!("0x1d09e36bc1db6b3e83298d8045cda770ca55eaeff1da0d44e684647653a1a185"),
    fixed_bytes!("0x266afaeab47b775c2275cde3248b68503f3079eca6461c1907fec9b979afe9ff"),
    fixed_bytes!("0x22794d6b26dd7398aa4f3c7d58ed5ea48f698ff4b229d21442846d8cd70959b1"),
    fixed_bytes!("0x05e208e2e76bcfe61cb39a79c0e263ee7874ba71cd64bc54e8bafd470055c6ef"),
    fixed_bytes!("0x26c093f627ffb8a25ab933cf64dd4f29dae2b103b48db3bf619f0dc39b298222"),
    fixed_bytes!("0x058676dab63180e26827fc2d2feccd6b191aa0e6589aa589398addb28e71a011"),
    fixed_bytes!("0x0f9ba00d2e0001bed485a0a1c2416e1aa2c86bf7c859c6707d0169170678f174"),
    fixed_bytes!("0x06fa06667c34201bcd5f6334de6b8c0b22b5f6bc57e401ed7660c40afd880b26"),
    fixed_bytes!("0x26ec3289eb146620b56807d58b3fae45adb7d7dfdc0a65194333e6dc2aa3de9e"),
    fixed_bytes!("0x2d2f60a05d456896411242de0eff23497c889f762e2eb5db0a07df329f452a92"),
];

#[cfg(feature = "contract")]
#[entrypoint]
#[storage]
pub struct IMT {
    depth: StorageU32,
    current_root_index: StorageU32,
    next_leaf_index: StorageU32,
    cached_subtrees: StorageMap<U32, StorageFixedBytes<32>>,
    roots: StorageArray<StorageFixedBytes<32>, 15>,
}

/* ======================================================================
 *                               Contract
 * ====================================================================== */
#[cfg(feature = "contract")]
#[public]
impl IMT {
    #[constructor]
    fn initialize(&mut self, depth: U32) -> Result<(), ContractErrors> {
        let depth_u32: u32 = u32::from_be_bytes(depth.to_be_bytes::<4>());
        if depth_u32 == 0 || depth_u32 >= 16 {
            return Err(ContractErrors::invalid_depth());
        }
        self.depth.set(depth);
        self.current_root_index.set(U32::from(0u32));
        self.next_leaf_index.set(U32::from(0u32));
        let init_root = Self::zeros_u32(depth_u32);
        self.roots.setter(U32::from(0u32)).unwrap().set(init_root);

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
        Ok(U32::from(next_idx_u32))
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

    fn zeros(&self, i: U256) -> FixedBytes<32> {
        let index_bytes = i.to_be_bytes::<32>();
        let index = u32::from_be_bytes([
            index_bytes[28],
            index_bytes[29],
            index_bytes[30],
            index_bytes[31],
        ]);
        Self::zeros_u32(index)
    }

    /* getters */
    fn get_depth(&self) -> U32 {
        self.depth.get()
    }

    fn get_current_root_index(&self) -> U32 {
        self.current_root_index.get()
    }

    fn get_next_leaf_index(&self) -> U32 {
        self.next_leaf_index.get()
    }

    fn get_root_from_root_index(&self, root_index: U32) -> FixedBytes<32> {
        self.roots.getter(root_index).unwrap().get()
    }
}

/* ======================================================================
 *                               INTERNAL HELPERS
 * ====================================================================== */
#[cfg(feature = "contract")]
impl IMT {
    fn hash_pair(&mut self, left: FixedBytes<32>, right: FixedBytes<32>) -> FixedBytes<32> {
        hash_two_fixed_bytes(left, right)
    }

    fn zeros_u32(i: u32) -> FixedBytes<32> {
        ZERO_LEAVES
            .get(i as usize)
            .copied()
            .expect("index out of bounds")
    }
}
