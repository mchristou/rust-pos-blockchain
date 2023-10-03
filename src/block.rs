use chrono::Utc;
use sha2::{Digest, Sha256};

#[derive(Debug, Default, Clone)]
pub struct Block {
    pub index: usize,
    pub hash: String,
    pub previous_hash: String,
    pub validator: String,
    timestamp: u64,
}

impl Block {
    pub fn new(index: usize, previous_hash: String, validator: String) -> Self {
        let block = Block {
            index,
            previous_hash,
            timestamp: Utc::now().timestamp() as u64,
            validator,
            hash: String::default(),
        };

        Block {
            hash: block.hash(),
            ..block
        }
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!(
            "{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, self.validator
        );

        hasher.update(data.as_bytes());

        format!("{:x}", hasher.finalize())
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block #{} [Hash: {}, Prev. Hash: {}, Validator: {}]",
            self.index, self.hash, self.previous_hash, self.validator
        )
    }
}
