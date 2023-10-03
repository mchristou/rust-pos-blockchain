use std::sync::{Arc, RwLock};

use crate::block::Block;

#[derive(Clone, Debug)]
pub struct Blockchain {
    chain: Arc<RwLock<Vec<Block>>>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();
        let genesis = Block::new(0, "".to_string(), "genesis".to_string());
        chain.push(genesis);

        Blockchain {
            chain: Arc::new(RwLock::new(chain)),
        }
    }

    pub fn add_block(&self, block: Block) -> Result<(), String> {
        if self.block_is_valid(&block) {
            self.chain
                .write()
                .map_err(|_| "Failed to acquire write lock")?
                .push(block);

            Ok(())
        } else {
            Err("Invalid block".to_string())
        }
    }

    pub fn last(&self) -> Option<Block> {
        let block = self.chain.read().ok()?.last().cloned();
        block
    }

    pub fn block_is_valid(&self, new_block: &Block) -> bool {
        if let Some(prev_block) = self.last() {
            prev_block.index + 1 == new_block.index
                && prev_block.hash == new_block.previous_hash
                && new_block.hash() == new_block.hash
        } else {
            false
        }
    }
}

impl std::fmt::Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain = self.chain.read().map_err(|_| std::fmt::Error)?;
        for block in chain.iter() {
            writeln!(f, "{}", block)?;
        }
        Ok(())
    }
}
