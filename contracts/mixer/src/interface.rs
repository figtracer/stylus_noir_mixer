pub use callable::*;

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface IMixer {
            function deposit(bytes32 commitment) external;
        }
    }
}
