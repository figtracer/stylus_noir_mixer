#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract IMTAbi {
        #[derive(Debug)]
        function insert(bytes32 leaf) external returns (uint index);
        function setHasher(address hasher) external;
        function isKnownRoot(bytes32 root) external view returns (bool known);
        function zeros(uint i) external view returns (bytes32 z);
    }
);
