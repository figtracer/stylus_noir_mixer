pub use callable::*;

mod callable {
    #![allow(missing_docs)]
    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface PoseidonInterface {
            function hash(uint[2] memory inputs) external view returns (uint hash);
        }
    }
}
