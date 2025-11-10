pub use callable::*;

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface IMixer {
            function deposit(bytes32 commitment) external;
        }

        interface IVerifier {
            function verify(bytes calldata _proof, bytes32[] calldata _public_inputs) external view returns (bool);
        }
    }
}
