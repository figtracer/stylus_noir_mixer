use stylus_sdk::prelude::*;
use stylus_sdk::alloy_sol_types::sol;

sol! {
	/* imt */
	error InvalidDepth();
	error TreeIsFull();

	/* mixer */
	error CommitmentAlreadyExists();
	error InvalidDenomination();
}

#[derive(SolidityError)]
pub enum ContractErrors {
	InvalidDepth(InvalidDepth),
	TreeIsFull(TreeIsFull),
	CommitmentAlreadyExists(CommitmentAlreadyExists),
	InvalidDenomination(InvalidDenomination),
}

impl ContractErrors {
	pub fn invalid_depth() -> Self { Self::InvalidDepth(InvalidDepth {}) }
	pub fn tree_is_full() -> Self { Self::TreeIsFull(TreeIsFull {}) }
	pub fn commitment_already_exists() -> Self { Self::CommitmentAlreadyExists(CommitmentAlreadyExists {}) }
	pub fn invalid_denomination() -> Self { Self::InvalidDenomination(InvalidDenomination {}) }
}


