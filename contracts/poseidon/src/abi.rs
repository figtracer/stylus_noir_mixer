use alloy::sol;
pub use callable::*;

sol!(
    #[sol(rpc)]
   contract PoseidonAbi {
        #[derive(Debug)]
        function hash(uint[2] memory inputs) external view returns (uint hash);
    }
);

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface PoseidonInterface {
            function hash(uint256[2] inputs) external view returns (uint256);
        }
    }
}
