pub use callable::*;

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface MixerInterface {
            function deposit(bytes32 commitment) external;
            function withdraw(bytes calldata proof, bytes32 root, bytes32 nullifier_hash, address recipient) external;
        }

        interface VerifierInterface {
            function verify(bytes calldata _proof, bytes32[] calldata _public_inputs) external view returns (bool);
        }
    }
}
