use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
    sync::mpsc,
    time::{self, Duration},
};

use crate::{block::Block, chain::Blockchain, validators::Validators};

pub async fn run_server(
    blockchain: Blockchain,
    validators: Validators,
    candidate_tx: mpsc::Sender<Block>,
) -> std::io::Result<()> {
    let transaction_tx = Arc::new(Mutex::new(Vec::<mpsc::Sender<()>>::new()));

    interval_task(transaction_tx.clone()).await;

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, _socket_addr) = listener.accept().await?;
        let candidate_tx = candidate_tx.clone();
        let blockchain = blockchain.clone();

        let validators_cloned = validators.clone();

        let (tr_tx, tr_rx) = mpsc::channel(1);
        transaction_tx.lock().unwrap().push(tr_tx);

        tokio::spawn(async move {
            handle_connection(stream, validators_cloned, candidate_tx, blockchain, tr_rx).await;
        });
    }
}

async fn interval_task(all_tx: Arc<Mutex<Vec<mpsc::Sender<()>>>>) {
    // simulate transactions
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            {
                interval.tick().await;

                let transaction_tx = all_tx.lock().unwrap();
                for tx in transaction_tx.iter() {
                    if let Err(e) = tx.try_send(()) {
                        eprintln!("{e}");
                    }
                }
            }
        }
    });
}

async fn handle_connection(
    mut stream: TcpStream,
    validators: Validators,
    candidate_tx: mpsc::Sender<Block>,
    blockchain: Blockchain,
    mut rx: mpsc::Receiver<()>,
) {
    let address = stream.peer_addr().unwrap().to_string();
    println!("Connection from {}", address);

    let (mut reader, mut writer) = stream.split();

    let _balance = read_from_stream(&mut writer, &mut reader, "Enter balance:\n").await;
    // TODO: check balance vs stake

    let validator = hash(&address);
    let amount = read_from_stream(&mut writer, &mut reader, "Enter new amount:\n").await;
    validators.insert(&validator, amount);

    if let Err(e) = writer
        .write_all(format!("Validator hash: {validator}\n").as_bytes())
        .await
    {
        println!("{e}");
    }

    if let Err(e) = writer
        .write_all(
            format!(
                "Last block:\n {}\n",
                blockchain.last().expect("Invalid chain")
            )
            .as_bytes(),
        )
        .await
    {
        println!("{e}");
    }

    loop {
        if (rx.recv().await).is_some() {
            let last_block = blockchain.last().expect("Invalid chain");
            let new_block = Block::new(last_block.index + 1, last_block.hash(), validator.clone());

            if let Err(e) = candidate_tx.send(new_block).await {
                println!("{e}");
            }

            // simulate broadcast of winning block
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Err(e) = writer
                .write_all(
                    format!(
                        "Last block:\n {}\n",
                        blockchain.last().expect("Invalid chain")
                    )
                    .as_bytes(),
                )
                .await
            {
                println!("{e}");
            }
        }
    }
}

fn hash(address: &str) -> String {
    let mut hasher = Sha256::new();
    let data = format!("{}{}", address, Utc::now().timestamp() as u64);

    hasher.update(data.as_bytes());

    format!("{:x}", hasher.finalize())
}

async fn read_from_stream(writer: &mut WriteHalf<'_>, reader: &mut ReadHalf<'_>, msg: &str) -> u32 {
    let mut buffer = [0; 1024];
    let mut value = String::new();

    if let Err(e) = writer.write_all(msg.as_bytes()).await {
        println!("{e}");
        return 0;
    }

    loop {
        while let Ok(n) = reader.read(&mut buffer).await {
            if n == 0 {
                break;
            }

            let message = &buffer[..n];
            if let Ok(s) = std::str::from_utf8(message) {
                value = s.to_string();
                break;
            }
        }

        match value.trim().parse() {
            Ok(num) => return num,
            Err(_) => {
                println!("Invalid value for {}", value);
                continue;
            }
        };
    }
}
