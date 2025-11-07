#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract IMTAbi {
        #[derive(Debug)]
        function insert(bytes32 leaf) external returns (uint32);
        function setHasher(address hasher) external;
        function isKnownRoot(bytes32 root) external view returns (bool known);
        function zeros(uint256 i) external view returns (bytes32 z);
        function getHasher() external view returns (address);
        function getDepth() external view returns (uint32);
        function getCurrentRootIndex() external view returns (uint32);
        function getNextLeafIndex() external view returns (uint32);
    }
);
