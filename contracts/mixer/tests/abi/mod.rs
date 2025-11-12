#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract MixerAbi {
        function deposit(bytes32 commitment) external;
        function withdraw(bytes calldata proof, bytes32 root, bytes32 nullifier_hash, address recipient) external;

        error InvalidDepth();
        error TreeIsFull();
        error CommitmentAlreadyExists();
        error InvalidDenomination();
        error NullifierHashAlreadyUsed();
        error InvalidRoot();
        error InvalidProof();

        #[derive(Debug, PartialEq)]
        event Deposit(bytes32 indexed commitment, uint32 index, uint256 timestamp);
        #[derive(Debug, PartialEq)]
        event Withdrawal(address indexed recipient, bytes32 indexed nullifier_hash);
    }
);

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
        function getRootFromRootIndex(uint32 root_index) external view returns (bytes32);
    }
);

sol!(
    #[sol(rpc)]
    contract PoseidonAbi {
        function hash(uint256[2] memory inputs) external view returns (uint256 hash);
    }
);
