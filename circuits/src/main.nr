use poseidon::poseidon2;
mod imt;

fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    /* private */
    nullifier: Field,
    secret: Field,
    /* the imt is going to have a depth of 20 in this case */
    merkle_proof: [Field; 20],
    is_even: [bool; 20],
) {
    /* compute commitment (hash nullifier and secret) */
    let commitment: Field = poseidon2::Poseidon2::hash([nullifier, secret], 2);

    /* compute nullifier hash to compare with the public nullifier hash */
    let computed_nullifier_hash: Field = poseidon2::Poseidon2::hash([nullifier], 1);
    assert(computed_nullifier_hash == nullifier_hash);

    let computed_root: Field = imt::compute_merkle_root(commitment, merkle_proof, is_even);
    assert(computed_root == root);
}
