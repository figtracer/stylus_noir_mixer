## stylus_noir_mixer

> Still in development. Might be broken somewhere.

Zero-knowledge mixer written in Rust using Arbitrum Stylus SDK

- Noir circuits (Poseidon2, depth-31 IMT path verification)
- Arbitrum Stylus Rust contracts (Poseidon hasher, Incremental Merkle Tree, Mixer)
- A Solidity UltraHonk verifier for on-chain proof verification (generated with bb write_solidity_verifier)
- Node scripts using bb.js and noir_js to generate commitments and proofs

This repository demonstrates end-to-end deposit flow primitives: commitment generation, Merkle inclusion proof creation, and Stylus contracts wiring for on-chain state and verification.

## Acknowledgements

- Cyfrin
- Noir (Aztec)
- Barretenberg (Aztec)
- Arbitrum Stylus SDK
- OpenZeppelin
