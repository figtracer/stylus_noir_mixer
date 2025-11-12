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
