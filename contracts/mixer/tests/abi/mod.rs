#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
   contract MixerAbi {
        #[derive(Debug)]
        function deposit(bytes32 commitment) external;
        function withdraw(bytes calldata proof, bytes32 root, bytes32 nullifier_hash, address recipient) external;
    }
);
