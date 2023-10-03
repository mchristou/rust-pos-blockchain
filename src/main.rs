use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use std::collections::HashSet;
use tokio::sync::mpsc;

mod block;
mod chain;
mod server;
mod validators;

use block::Block;
use chain::Blockchain;
use validators::Validators;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let blockchain = Blockchain::new();
    let validators = Validators::new();
    let (candidate_tx, mut candidate_rx) = mpsc::channel::<Block>(16);
    let blockchain_cloned = blockchain.clone();
    let validators_cloned = validators.clone();

    tokio::spawn(async move {
        let mut temp_blocks: Vec<_> = vec![];
        let mut received_validators = HashSet::new();

        loop {
            if let Some(candidate_block) = candidate_rx.recv().await {
                temp_blocks.push(candidate_block.clone());
                received_validators.insert(candidate_block.validator.clone());
            }

            if received_validators.len() == validators_cloned.len() {
                let mut lottery_pool = vec![];

                for block in &temp_blocks {
                    if !lottery_pool.contains(&block.validator) {
                        let validator_stake = validators_cloned
                            .stake(&block.validator)
                            .unwrap_or_default();
                        for _ in 0..validator_stake {
                            lottery_pool.push(block.validator.clone());
                        }
                    }
                }

                let mut rng = StdRng::from_entropy();
                lottery_pool.shuffle(&mut rng);
                let index = rng.gen_range(0..lottery_pool.len());
                let winner = &lottery_pool[index];

                for block in &temp_blocks {
                    if &block.validator == winner {
                        blockchain_cloned
                            .add_block(block.clone())
                            .expect("Failed to add block");
                    }
                }

                println!(
                    "Proposed blocks: {}, winning validator: {}",
                    temp_blocks.len(),
                    winner
                );

                temp_blocks.clear();
                received_validators.clear();
            }
        }
    });

    server::run_server(blockchain, validators.clone(), candidate_tx).await?;

    Ok(())
}
