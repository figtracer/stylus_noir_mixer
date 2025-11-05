pub use callable::*;
use stylus_sdk::alloy_sol_types::sol;

sol! {
    interface PoseidonAbi {
        function hash(uint256[2] inputs) external view returns (uint256);
    }
}

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface PoseidonInterface {
            function hash(uint256[2] inputs) external view returns (uint256);
        }
    }
}
