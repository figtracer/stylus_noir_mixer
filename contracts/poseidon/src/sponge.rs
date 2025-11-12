pub use sponge::hash;

use openzeppelin_crypto::{
    arithmetic::uint::U256, field::instance::FpBN256, poseidon2::params::PoseidonParams,
};

use crate::params::StylusBN256Params;

const STATE_WIDTH: usize = StylusBN256Params::T;
const RATE: usize = STATE_WIDTH - StylusBN256Params::CAPACITY;

mod sponge {
    use super::*;

    pub fn hash(inputs: &[FpBN256], std_input_length: usize, is_variable_length: bool) -> FpBN256 {
        let iv = generate_iv(inputs.len());
        let mut sponge = Sponge::new(iv);

        for (index, input) in inputs.iter().copied().enumerate() {
            if index < std_input_length {
                sponge.absorb(input);
            }
        }

        if is_variable_length {
            sponge.absorb(FpBN256::ONE);
        }

        sponge.squeeze()
    }

    fn generate_iv(input_length: usize) -> FpBN256 {
        let mut iv = U256::from(input_length as u64);
        iv <<= 64;
        FpBN256::from_bigint(iv)
    }

    struct Sponge {
        state: [FpBN256; STATE_WIDTH],
        cache: [FpBN256; RATE],
        cache_size: usize,
        squeeze_mode: bool,
    }

    impl Sponge {
        fn new(iv: FpBN256) -> Self {
            let mut state = [FpBN256::ZERO; STATE_WIDTH];
            state[RATE] = iv;

            Self {
                state,
                cache: [FpBN256::ZERO; RATE],
                cache_size: 0,
                squeeze_mode: false,
            }
        }

        fn absorb(&mut self, input: FpBN256) {
            if self.squeeze_mode {
                panic!("cannot absorb while squeezing");
            }

            if self.cache_size == RATE {
                self.perform_duplex();
                self.cache[0] = input;
                self.cache_size = 1;
            } else {
                self.cache[self.cache_size] = input;
                self.cache_size += 1;
            }
        }

        fn squeeze(&mut self) -> FpBN256 {
            if !self.squeeze_mode {
                let new_elements = self.perform_duplex();
                self.cache = new_elements;
                self.cache_size = RATE;
                self.squeeze_mode = true;
            } else if self.cache_size == 0 {
                let new_elements = self.perform_duplex();
                self.cache = new_elements;
                self.cache_size = RATE;
            }

            let result = self.cache[0];

            for i in 1..self.cache_size {
                self.cache[i - 1] = self.cache[i];
            }

            self.cache_size -= 1;
            self.cache[self.cache_size] = FpBN256::ZERO;

            result
        }

        fn perform_duplex(&mut self) -> [FpBN256; RATE] {
            for index in self.cache_size..RATE {
                self.cache[index] = FpBN256::ZERO;
            }

            for index in 0..RATE {
                self.state[index] += self.cache[index];
            }

            permutation(&mut self.state);

            let mut result = [FpBN256::ZERO; RATE];
            for index in 0..RATE {
                result[index] = self.state[index];
            }

            result
        }
    }

    fn permutation(state: &mut [FpBN256; STATE_WIDTH]) {
        matrix_multiplication_4x4(state);

        let full_rounds_half = StylusBN256Params::ROUNDS_F / 2;
        let partial_rounds_end = full_rounds_half + StylusBN256Params::ROUNDS_P;
        let total_rounds = StylusBN256Params::ROUNDS_F + StylusBN256Params::ROUNDS_P;

        for round in 0..full_rounds_half {
            add_round_constants(state, StylusBN256Params::ROUND_CONSTANTS[round]);
            s_box(state);
            matrix_multiplication_4x4(state);
        }

        for round in full_rounds_half..partial_rounds_end {
            state[0] += StylusBN256Params::ROUND_CONSTANTS[round][0];
            state[0] = single_box(state[0]);
            internal_m_multiplication(state);
        }

        for round in partial_rounds_end..total_rounds {
            add_round_constants(state, StylusBN256Params::ROUND_CONSTANTS[round]);
            s_box(state);
            matrix_multiplication_4x4(state);
        }
    }

    fn add_round_constants(state: &mut [FpBN256; STATE_WIDTH], constants: &[FpBN256]) {
        for (value, constant) in state.iter_mut().zip(constants.iter()) {
            *value += *constant;
        }
    }

    fn s_box(state: &mut [FpBN256; STATE_WIDTH]) {
        for value in state.iter_mut() {
            *value = single_box(*value);
        }
    }

    fn single_box(x: FpBN256) -> FpBN256 {
        let x2 = x * x;
        let x4 = x2 * x2;
        x4 * x
    }

    fn internal_m_multiplication(state: &mut [FpBN256; STATE_WIDTH]) {
        let sum = state
            .iter()
            .copied()
            .fold(FpBN256::ZERO, |acc, value| acc + value);

        for (value, diagonal) in state
            .iter_mut()
            .zip(StylusBN256Params::MAT_INTERNAL_DIAG_M_1.iter())
        {
            *value *= *diagonal;
            *value += sum;
        }
    }

    fn matrix_multiplication_4x4(state: &mut [FpBN256; STATE_WIDTH]) {
        let a = state[0];
        let b = state[1];
        let c = state[2];
        let d = state[3];

        let t0 = a + b;
        let t1 = c + d;
        let t2 = (b + b) + t1;
        let t3 = (d + d) + t0;
        let t4 = ((t1 + t1) + (t1 + t1)) + t3;
        let t5 = ((t0 + t0) + (t0 + t0)) + t2;
        let t6 = t3 + t5;
        let t7 = t2 + t4;

        state[0] = t6;
        state[1] = t5;
        state[2] = t7;
        state[3] = t4;
    }
}

#[cfg(test)]
mod tests {
    use super::hash;
    use alloy_primitives::{hex, U256 as AlloyU256};
    use openzeppelin_crypto::arithmetic::uint::U256;
    use openzeppelin_crypto::field::instance::FpBN256;

    fn fp_from_u256(value: AlloyU256) -> FpBN256 {
        FpBN256::from_bigint(U256::from(value))
    }

    #[test]
    fn poseidon_hash_two_inputs_matches_vector() {
        let inputs = [
            fp_from_u256(AlloyU256::from(123u64)),
            fp_from_u256(AlloyU256::from(123_456u64)),
        ];

        let result = hash(&inputs, inputs.len(), false);
        let expected = fp_from_u256(AlloyU256::from_be_slice(&hex!(
            "1f24fc186957171704ab4ddf424d2830a3f5d04910752a162cd93487ebdc634d"
        )));

        assert_eq!(result, expected);
    }

    #[test]
    fn poseidon_hash_known_vector_matches() {
        let inputs = [
            fp_from_u256(AlloyU256::from_be_slice(&hex!(
                "1f8fb4ad3f03c2e36e1fcf77a43e41b55f01c37231981130f687b0019df78374"
            ))),
            fp_from_u256(AlloyU256::from_be_slice(&hex!(
                "080104bc5c9cc4a6c922cf39f3a7f3e8820d988904094e3d5087c0bc3e93e3bc"
            ))),
        ];

        let result = hash(&inputs, inputs.len(), false);
        let expected = fp_from_u256(AlloyU256::from_be_slice(&hex!(
            "083d10323077fed15f77b82c26a7f28ae8ce785a19716a26e2c96d695a8effae"
        )));

        assert_eq!(result, expected);
    }
}
