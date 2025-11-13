import { Barretenberg, Fr } from "@aztec/bb.js";
import { ethers } from "ethers";

export default async function generateCommitment(): Promise<string> {
  const bb = await Barretenberg.new();
  const nullifier = Fr.random();
  const secret = Fr.random();
  const commitment: Fr = await bb.poseidon2Hash([nullifier, secret]);

  const toHexString = (fr: Fr) => ethers.hexlify(fr.toBuffer());

  const result = {
    commitment: toHexString(commitment),
    nullifier: toHexString(nullifier),
    secret: toHexString(secret),
  };

  return JSON.stringify(result);
}

(async () => {
  generateCommitment()
    .then((result) => {
      process.stdout.write(result);
      process.exit(0);
    })
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
})();
