import { Barretenberg, Fr } from "@aztec/bb.js";

async function hashLeftRight(left, right) {
  const bb = await Barretenberg.new();
  const frLeft = Fr.fromString(left);
  const frRight = Fr.fromString(right);
  const hash = await bb.poseidon2Hash([frLeft, frRight]);
  return hash.toString();
}

export class PoseidonTree {
  constructor(levels, zeros) {
    if (zeros.length < levels + 1) {
      throw new Error(
        "Not enough zero values provided for the given tree height."
      );
    }
    this.levels = levels;
    this.hashLeftRight = hashLeftRight;
    this.storage = new Map();
    this.zeros = zeros;
    this.totalLeaves = 0;
  }

  async init(defaultLeaves = []) {
    if (defaultLeaves.length > 0) {
      this.totalLeaves = defaultLeaves.length;

      defaultLeaves.forEach((leaf, index) => {
        this.storage.set(PoseidonTree.indexToKey(0, index), leaf);
      });

      for (let level = 1; level <= this.levels; level++) {
        const numNodes = Math.ceil(this.totalLeaves / 2 ** level);
        for (let i = 0; i < numNodes; i++) {
          const left =
            this.storage.get(PoseidonTree.indexToKey(level - 1, 2 * i)) ||
            this.zeros[level - 1];
          const right =
            this.storage.get(PoseidonTree.indexToKey(level - 1, 2 * i + 1)) ||
            this.zeros[level - 1];
          const node = await this.hashLeftRight(left, right);
          this.storage.set(PoseidonTree.indexToKey(level, i), node);
        }
      }
    }
  }

  static indexToKey(level, index) {
    return `${level}-${index}`;
  }

  getIndex(leaf) {
    for (const [key, value] of this.storage.entries()) {
      if (value === leaf && key.startsWith("0-")) {
        return parseInt(key.split("-")[1]);
      }
    }
    return -1;
  }

  root() {
    return (
      this.storage.get(PoseidonTree.indexToKey(this.levels, 0)) ||
      this.zeros[this.levels]
    );
  }

  proof(index) {
    const leaf = this.storage.get(PoseidonTree.indexToKey(0, index));
    if (!leaf) throw new Error("leaf not found");

    const pathElements = [];
    const pathIndices = [];

    this.traverse(index, (level, currentIndex, siblingIndex) => {
      const sibling =
        this.storage.get(PoseidonTree.indexToKey(level, siblingIndex)) ||
        this.zeros[level];
      pathElements.push(sibling);
      pathIndices.push(currentIndex % 2);
    });

    return {
      root: this.root(),
      pathElements,
      pathIndices,
      leaf,
    };
  }

  async insert(leaf) {
    const index = this.totalLeaves;
    await this.update(index, leaf, true);
    this.totalLeaves++;
  }

  async update(index, newLeaf, isInsert = false) {
    if (!isInsert && index >= this.totalLeaves) {
      throw Error("Use insert method for new elements.");
    } else if (isInsert && index < this.totalLeaves) {
      throw Error("Use update method for existing elements.");
    }

    const keyValueToStore = [];
    let currentElement = newLeaf;

    await this.traverseAsync(
      index,
      async (level, currentIndex, siblingIndex) => {
        const sibling =
          this.storage.get(PoseidonTree.indexToKey(level, siblingIndex)) ||
          this.zeros[level];
        const [left, right] =
          currentIndex % 2 === 0
            ? [currentElement, sibling]
            : [sibling, currentElement];
        keyValueToStore.push({
          key: PoseidonTree.indexToKey(level, currentIndex),
          value: currentElement,
        });
        currentElement = await this.hashLeftRight(left, right);
      }
    );

    keyValueToStore.push({
      key: PoseidonTree.indexToKey(this.levels, 0),
      value: currentElement,
    });
    keyValueToStore.forEach(({ key, value }) => this.storage.set(key, value));
  }

  traverse(index, fn) {
    let currentIndex = index;
    for (let level = 0; level < this.levels; level++) {
      const siblingIndex =
        currentIndex % 2 === 0 ? currentIndex + 1 : currentIndex - 1;
      fn(level, currentIndex, siblingIndex);
      currentIndex = Math.floor(currentIndex / 2);
    }
  }

  async traverseAsync(index, fn) {
    let currentIndex = index;
    for (let level = 0; level < this.levels; level++) {
      const siblingIndex =
        currentIndex % 2 === 0 ? currentIndex + 1 : currentIndex - 1;
      await fn(level, currentIndex, siblingIndex);
      currentIndex = Math.floor(currentIndex / 2);
    }
  }
}

const ZERO_VALUES = [
  "0x168db4aa1d4e4bf2ee46eb882e1c38a7de1a4da47e17b207a5494a14605ae38e",
  "0x257a568bdc9cc663b2cf123f7d7b6c5eedd5a312d2792305352e09f1733a56b5",
  "0x25b9b4ff326c7783ce7a3ae1503dce4552211bdfb510808e215f4227da087023",
  "0x0aa6931cdcc4482ac0a053cf28a380154ce6500cc02087ea9c8b71ffe597ea59",
  "0x20cb91532baf018f130fc336438e923c9f2de935efdd4325a8c7eda10d5c5520",
  "0x1ca38bd416b196d58f59133a826b64ec9f697e854ea8f10b9337c74365e79068",
  "0x1d09e36bc1db6b3e83298d8045cda770ca55eaeff1da0d44e684647653a1a185",
  "0x266afaeab47b775c2275cde3248b68503f3079eca6461c1907fec9b979afe9ff",
  "0x22794d6b26dd7398aa4f3c7d58ed5ea48f698ff4b229d21442846d8cd70959b1",
  "0x05e208e2e76bcfe61cb39a79c0e263ee7874ba71cd64bc54e8bafd470055c6ef",
  "0x26c093f627ffb8a25ab933cf64dd4f29dae2b103b48db3bf619f0dc39b298222",
  "0x058676dab63180e26827fc2d2feccd6b191aa0e6589aa589398addb28e71a011",
  "0x0f9ba00d2e0001bed485a0a1c2416e1aa2c86bf7c859c6707d0169170678f174",
  "0x06fa06667c34201bcd5f6334de6b8c0b22b5f6bc57e401ed7660c40afd880b26",
  "0x26ec3289eb146620b56807d58b3fae45adb7d7dfdc0a65194333e6dc2aa3de9e",
  "0x2d2f60a05d456896411242de0eff23497c889f762e2eb5db0a07df329f452a92",
  "0x1ee903a4eac57310c624c0e30f2bd083eb68a595306df83b1111db0fffce45ea",
  "0x05f96e491710c7e1d65207b36e0031c1de403eb32753de2489e8abce4c2e86ff",
  "0x2375b170da8f212cf2b23538990cb6a2e319c50eee555a3fcbed25946326be6c",
  "0x14307dca3f2b6224ff19c5c0a19129c5fa79d48c645ebb1c5302cb41a131e72a",
  "0x051e91aeea86b05dcd2b5218126fb3cf3990c81d53f0947028a933026eb94b3a",
];

export async function merkleTree(leaves) {
  const TREE_HEIGHT = 10;
  const tree = new PoseidonTree(TREE_HEIGHT, ZERO_VALUES);

  // Initialize tree with no leaves (all zeros)
  await tree.init();

  // Insert some leaves (from input)
  for (const leaf of leaves) {
    await tree.insert(leaf);
  }

  return tree;
}
