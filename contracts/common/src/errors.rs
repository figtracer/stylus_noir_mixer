extern crate alloc;
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::prelude::*;

sol! {
    /* imt */
    error InvalidDepth();
    error TreeIsFull();

    /* mixer */
    error CommitmentAlreadyExists();
    error InvalidDenomination();
    error NullifierHashAlreadyUsed();
    error InvalidRoot();
    error InvalidProof();
}

#[derive(SolidityError)]
pub enum ContractErrors {
    InvalidDepth(InvalidDepth),
    TreeIsFull(TreeIsFull),
    CommitmentAlreadyExists(CommitmentAlreadyExists),
    InvalidDenomination(InvalidDenomination),
    NullifierHashAlreadyUsed(NullifierHashAlreadyUsed),
    InvalidRoot(InvalidRoot),
    InvalidProof(InvalidProof),
}

impl ContractErrors {
    pub fn invalid_depth() -> Self {
        Self::InvalidDepth(InvalidDepth {})
    }
    pub fn tree_is_full() -> Self {
        Self::TreeIsFull(TreeIsFull {})
    }
    pub fn commitment_already_exists() -> Self {
        Self::CommitmentAlreadyExists(CommitmentAlreadyExists {})
    }
    pub fn invalid_denomination() -> Self {
        Self::InvalidDenomination(InvalidDenomination {})
    }
    pub fn nullifier_hash_already_used() -> Self {
        Self::NullifierHashAlreadyUsed(NullifierHashAlreadyUsed {})
    }
    pub fn invalid_root() -> Self {
        Self::InvalidRoot(InvalidRoot {})
    }
    pub fn invalid_proof() -> Self {
        Self::InvalidProof(InvalidProof {})
    }
}
