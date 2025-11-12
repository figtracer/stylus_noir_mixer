import { Barretenberg, Fr } from "@aztec/bb.js";
import { ethers } from "ethers";

export default async function generateCommitment(): Promise<string> {
  const bb = await Barretenberg.new();
  const nullifier = Fr.random();
  const secret = Fr.random();
  console.log("nullifier: ", nullifier.toString());
  console.log("secret: ", secret.toString());
  const commitment: Fr = await bb.poseidon2Hash([nullifier, secret]);

  const toHexString = (value: string) => {
    try {
      return ethers.hexlify(Fr.fromString(value).toBuffer());
    } catch {
      return ethers.hexlify(ethers.getBytes(value));
    }
  };

  const commitmentHex = toHexString(commitment.toString());
  const nullifierHex = toHexString(nullifier.toString());
  const secretHex = toHexString(secret.toString());

  const result = {
    commitment: commitmentHex,
    nullifier: nullifierHex,
    secret: secretHex,
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
