use crate::Block;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread::sleep;
use std::time::Duration;

const URL: &str = "http://localhost:8080";
const REQUEST_PAUSE_IN_SECONDS: u64 = 1;

pub struct NetworkConnector {
    // channel to send new blocks received from the server
    tx: SyncSender<Vec<Block>>,
    // channel to receive a new block from the miner.
    rx: Receiver<Block>,
}

impl NetworkConnector {
    pub fn new(tx: SyncSender<Vec<Block>>, rx: Receiver<Block>) -> Self {
        NetworkConnector { tx, rx }
    }

    pub fn sync(&mut self) -> reqwest::Result<()> {
        let mut fullurlpost: String = URL.to_owned();
        fullurlpost.push_str("/postblock");
        let client = reqwest::blocking::Client::new();

        loop {
            // Check if the miner wants us to send any block
            if let Ok(block) = self.rx.try_recv() {
                let resp = client.post(&fullurlpost).json(&block).send()?;

                if resp.status().is_client_error() {
                    println!("An error occured: {:?}", resp.text()?)
                }
            }
            // Recover all the blocks known by the server and send
            // them to the miner.
            match get_blocks() {
                // If the miner did not consume previous blocks; we drop them
                // and retry in one second.
                Ok(blocks) => self.tx.try_send(blocks).unwrap_or(()),
                Err(e) => println!("Request failed {:?}", e),
            };

            sleep(Duration::from_secs(REQUEST_PAUSE_IN_SECONDS));
        }
    }
}

pub fn get_blocks() -> reqwest::Result<Vec<Block>> {
    let mut fullurl: String = URL.to_owned();
    fullurl.push_str("/blocks");
    let blocks: Vec<Block> = reqwest::blocking::get(fullurl)?.json()?;
    Ok(blocks)
}
