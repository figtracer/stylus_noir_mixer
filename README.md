## stylus_mixer

> Might be broken somewhere.

Zero-knowledge mixer written in Rust using Arbitrum Stylus SDK

- Noir circuits (Poseidon2, depth-31 IMT path verification)
- Arbitrum Stylus Rust contracts (Poseidon hasher, Incremental Merkle Tree, Mixer)
- A Solidity UltraHonk verifier for on-chain proof verification (generated with bb write_solidity_verifier)
- Node scripts using bb.js and noir_js to generate commitments and proofs

## Acknowledgements

- Cyfrin
- Noir (Aztec)
- Barretenberg (Aztec)
- Arbitrum Stylus SDK
- OpenZeppelin
