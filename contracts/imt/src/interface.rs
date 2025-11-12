pub use callable::*;

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface IMTInterface {
            function insert(bytes32 leaf) external returns (uint32);
            function isKnownRoot(bytes32 root) external view returns (bool);
            function zeros(uint256 i) external view returns (bytes32);
            function getDepth() external view returns (uint32);
            function getCurrentRootIndex() external view returns (uint32);
            function getNextLeafIndex() external view returns (uint32);
            function getRootFromRootIndex(uint32 root_index) external view returns (bytes32);
        }
    }
}
