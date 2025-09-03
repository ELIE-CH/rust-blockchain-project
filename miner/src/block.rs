use rand::{RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const DIFFICULTY: u32 = 10;

#[derive(Default)]
pub struct BlockHasher {
    id: u64,
}

// @Student Remove if you're not using it.
impl std::hash::Hasher for BlockHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.id
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.id = id;
    }

    #[inline]
    fn write(&mut self, _: &[u8]) {
        // The id will always be a u64, so write_u64 will always be used, and this will never be
        // used. We need this definition due to the trait.
        unimplemented!()
    }
}

// @Student Remove if you're not using it.
pub type BlockIdHasher = std::hash::BuildHasherDefault<BlockHasher>;

// @Student Remove if you're not using it.
pub type BlockHashSet = HashSet<u64, BlockIdHasher>;


#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Block {
    /// Hash of the parent block
    pub parent_hash: Vec<u8>,
    /// Miner's (unique) identity. We don't use asymmetric cryptography for this simple exercise.
    pub miner: String,
    /// Random value such the hash value of this block is valid.
    pub nonce: u64,
    /// Dancemove chosen by the miner. That's the very strong incentive explaining
    /// why everyone one wants to mine on this blockchain.
    pub dancemove: DanceMove,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum DanceMove {
    #[default]
    Y = 1,
    M = 2,
    C = 3,
    A = 4,
}




impl Block {
    pub fn new(parent_hash: Vec<u8>, miner: String, nonce: u64, dancemove: DanceMove) -> Self {
        Block{
            parent_hash,
            miner,
            nonce,
            dancemove
        }
    }

    /// Computes the hash of self
    pub fn hash_block(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        hasher.update(&self.parent_hash);

        hasher.update(self.miner.as_bytes());

        hasher.update(&self.nonce.to_le_bytes());

        hasher.update(&[self.dancemove as u8]);

        hasher.finalize().into()
    }

    /// Solves the block finding a nonce that hashes the block to
    /// a hash value starting with `difficulty` bits set to 0. Returns the
    /// hash value of the block stored in a Vec.
    pub fn solve_block<R: RngCore>(
        &mut self,
        rng: &mut R,
        difficulty: u32,
        max_iteration: Option<u64>,
    ) -> Option<Vec<u8>> {
        for _ in 0..max_iteration.unwrap_or(u64::MAX) {
            self.nonce = rng.next_u64();
            let hash = self.hash_block();

            if self.pow_check(&hash, difficulty) {
                return Some(hash.to_vec());
            }
        }
        None
    }

    /// Checks if the proof of work is correct
    pub fn pow_check(&self, hash: &[u8], difficulty: u32) -> bool {
        let mut leading_bits = 0;

        for byte in hash {
            if *byte == 0 {
                leading_bits += 8;
            } else {
                leading_bits += byte.leading_zeros();
                break;
            }
        }

        leading_bits >= difficulty
    }

    pub fn is_genesis(&self, _difficulty: u32) -> bool {

        self.parent_hash.is_empty() && self.miner == "Genesis".to_string()
    }
}

impl crate::simpletree::Parenting for Block {
    fn is_parent(&self, parent_id: &[u8]) -> bool {
        self.hash_block().eq(parent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;

    #[test]
    fn test_pow_check() {
        let block = Block {
            parent_hash: vec![],
            miner: "test".to_string(),
            nonce: 0,
            dancemove: DanceMove::C,
        };

        // Test case where hash has sufficient leading zeros
        let hash_with_zeros = vec![0x00, 0x00, 0x00, 0xFF];
        assert!(block.pow_check(&hash_with_zeros, 24));

        // Test case with insufficient zeros
        let hash_without_zeros = vec![0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!block.pow_check(&hash_without_zeros, 1));

        // Test edge case (difficulty = 0)
        assert!(block.pow_check(&hash_without_zeros, 0));
    }

    #[test]
    fn test_solve_block() {
        let mut block = Block {
            parent_hash: vec![],
            miner: "test".to_string(),
            nonce: 0,
            dancemove: DanceMove::Y,
        };

        // Use a seeded Rng for deterministic testing
        let mut rng = StdRng::seed_from_u64(42);
        block.nonce = rng.random();

        for difficulty in 5..10 {
            if difficulty % 2 == 0 {
                block.dancemove = DanceMove::A;
            } else {
                block.dancemove = DanceMove::M;
            }

            let hash = block.solve_block(&mut rng, difficulty, None).unwrap();

            // Ensure the solved hash meets the difficulty
            assert!(block.pow_check(&hash, difficulty));

            // Ensure nonce changed
            assert_ne!(block.nonce, 0);
        }
    }

    #[test]
    fn test_new_genesis() {
        let mut genesis = Block::new(Vec::new(), "Genesis".to_string(), 42, DanceMove::C);
        let mut rng = StdRng::seed_from_u64(42);
        genesis.nonce = rng.random();
        genesis.solve_block(&mut rng, 10, None).unwrap();
        assert!(genesis.is_genesis(10));
    }
}
