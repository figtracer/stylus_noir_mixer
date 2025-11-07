#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract PoseidonAbi {
        #[derive(Debug)]
        function hash(uint256[2] memory inputs) external view returns (uint256 hash);
    }
);
