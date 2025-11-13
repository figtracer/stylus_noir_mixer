import { Barretenberg, Fr, UltraHonkBackend } from "@aztec/bb.js";
import { ethers } from "ethers";
import { merkleTree } from "./merkleTree.js";
import { Noir } from "@noir-lang/noir_js";
import path from "path";
import fs from "fs";

const circuit = JSON.parse(
  fs.readFileSync(
    path.resolve(__dirname, "../../circuits/target/circuits.json"),
    "utf8"
  )
);

export default async function generateProof() {
  const bb = await Barretenberg.new();

  const inputs = process.argv.slice(2);

  const nullifier = Fr.fromString(inputs[0]);
  const secret = Fr.fromString(inputs[1]);
  const nullifierHash = await bb.poseidon2Hash([nullifier]);
  const leaves = inputs.slice(3);

  const tree = await merkleTree(leaves);
  const commitment = await bb.poseidon2Hash([nullifier, secret]);
  const merkleProof = tree.proof(tree.getIndex(commitment.toString()));

  try {
    const noir = new Noir(circuit);
    const honk = new UltraHonkBackend(circuit.bytecode, { threads: 1 });
    const input = {
      root: merkleProof.root,
      nullifier_hash: nullifierHash.toString(),
      recipient: inputs[2],
      nullifier: nullifier.toString(),
      secret: secret.toString(),
      merkle_proof: merkleProof.pathElements.map((i) => i.toString()),
      is_even: merkleProof.pathIndices.map((i) => i % 2 == 0),
    };
    const { witness } = await noir.execute(input);

    const { proof, publicInputs } = await honk.generateProof(witness, {
      keccak: true,
    });

    const toHexString = (value: string) => {
      try {
        return ethers.hexlify(Fr.fromString(value).toBuffer());
      } catch {
        return ethers.hexlify(ethers.getBytes(value));
      }
    };

    const proofHex = ethers.hexlify(proof);
    const publicInputsHex = publicInputs.map((value: string) =>
      toHexString(value)
    );

    const result = {
      proof: proofHex,
      publicInputs: publicInputsHex,
    };

    return JSON.stringify(result);
  } catch (error) {
    console.log(error);
    throw error;
  }
}

(async () => {
  generateProof()
    .then((result) => {
      process.stdout.write(result);
      process.exit(0);
    })
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
})();
