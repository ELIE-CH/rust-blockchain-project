use block::Block;
use block::BlockHashSet;
use block::DanceMove;
use block::DIFFICULTY;
use clap::{Parser, Subcommand};
use network::NetworkConnector;
use simpletree::TreeNode;
use std::fmt;
use std::sync::mpsc;
use std::thread;
use rand::Rng;
use rand::rngs::ThreadRng;

const MY_NAME: &str = "miner1";

#[derive(Default, Debug)]
struct Blockchain {
    /// The blockchain is represented as a simple tree with no
    /// parent pointer.
    blocks: TreeNode<Block>,
}

impl Blockchain {
    /// Creates a new Blockchain from the provided genesis
    /// block and vector of valid blocks.
    pub fn new_from_genesis_and_vec(
        genesis: Block,
        blocks: Vec<Block>,
    ) -> (Self, Vec<Block>) {
        let mut tree = TreeNode::new(genesis);
        let mut remaining_blocks = blocks;
        let mut invalid_blocks = vec![];
        let mut blockids = BlockHashSet::default();


        let mut inserted_some = true;

        while inserted_some {
            inserted_some = false;
            let mut still_remaining = Vec::new();

            for block in remaining_blocks {
                if blockids.contains(&block.nonce) {
                    invalid_blocks.push(block);
                    continue;
                }

                if let Some(parent) = tree.look_for_parent(&block.parent_hash) {
                    parent.insert(block.clone());
                    blockids.insert(block.nonce);
                    inserted_some = true;
                } else {
                    still_remaining.push(block);
                }
            }

            remaining_blocks = still_remaining;
        }

        remaining_blocks.extend(invalid_blocks);

        (Blockchain { blocks: tree }, remaining_blocks)
    }

    fn print_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        node: &TreeNode<Block>,
        prefixes: &mut Vec<bool>,
    ) -> fmt::Result {
        // Print the current node
        if !prefixes.is_empty() {
            // Print connecting lines from parent
            for &is_last in &prefixes[..prefixes.len() - 1] {
                write!(f, "{}", if is_last { "    " } else { "│   " })?;
            }

            // Print the appropriate connector
            let is_last = *prefixes.last().unwrap();
            write!(f, "{}", if is_last { "└── " } else { "├── " })?;
        }

        // Print the block info
        let block = node.value();
        writeln!(f, "{} (nonce: {})", block.miner, block.nonce)?;

        // Recursively print children
        let child_count = node.children().len();
        for (i, child) in node.children().iter().enumerate() {
            prefixes.push(i == child_count - 1); // true if this is the last child
            self.print_tree(f, child, prefixes)?;
            prefixes.pop();
        }

        Ok(())
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_tree(f, &self.blocks, &mut Vec::new())
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    action: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Mine {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,
        #[arg(short, default_value_t = String::from(MY_NAME))]
        miner_name: String,
        #[arg(long)]
        max_iter: Option<u64>,
    },
    Print {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,
    },
}

fn mine(difficulty: &u32, miner_name: &String, max_iter: &Option<u64>) {
    // Create communication channels for the network
    let (tx_net_send, rx_net) = mpsc::sync_channel(1);
    let (tx_net, rx_net_ctrl) = mpsc::channel();

    // Network thread for synchronization
    thread::spawn(move || {
        let mut net = NetworkConnector::new(tx_net_send, rx_net_ctrl);
        net.sync().expect("Network failure");
    });

    let mut rng: ThreadRng = rand::rng();

    loop {
        let received = match rx_net.recv() {
            Ok(blocks) => blocks,
            Err(_) => {
                eprintln!("Failed to receive from network.");
                continue;
            }
        };

        // Search or create a genesis block
        let genesis = received.iter()
            .find(|b| b.is_genesis(*difficulty))
            .cloned()
            .unwrap_or_else(|| {
                let mut block = Block::new(vec![], "Genesis".to_string(), 0, random_dancemove(&mut rng));
                block.solve_block(&mut rng, *difficulty, *max_iter);
                tx_net.send(block.clone()).expect("Failed to send genesis block");
                block
            });

        let (chain, _) = Blockchain::new_from_genesis_and_vec(genesis.clone(), received);

        // Find the deepest leaf with the smallest nonce
        let leaf = chain.blocks
            .deepest_leafs()
            .into_iter()
            .min_by_key(|b| b.value().nonce)
            .unwrap()
            .value()
            .clone();

        // Create and mine a new block
        let mut new_block = Block::new(
            leaf.hash_block().to_vec(),
            miner_name.to_string(),
            0,
            random_dancemove(&mut rng),
        );
        new_block.solve_block(&mut rng, *difficulty, *max_iter);

        tx_net.send(new_block).expect("Failed to send block");

        println!("Current blockchain state:\n{}", chain);
    }
}

fn random_dancemove(rng: &mut ThreadRng) -> DanceMove {
    match rng.random_range(0..4) {
        0 => DanceMove::Y,
        1 => DanceMove::M,
        2 => DanceMove::C,
        _ => DanceMove::A,
    }
}





fn main() {

    let args = Args::parse();

    match &args.action {
        Some(Commands::Mine {
            difficulty,
            miner_name,
            max_iter,
        }) => {
            mine(difficulty, miner_name, max_iter);
        }

        Some(Commands::Print { difficulty: _ }) => {
            let (tx_net_send, rx_from_net) = mpsc::sync_channel(1);
            let (_tx_to_net, rx_for_net) = mpsc::channel();

            thread::spawn(move || {
                let mut net = NetworkConnector::new(tx_net_send, rx_for_net);
                net.sync().expect("Network synchronization failed");
            });

            let received_blocks = match rx_from_net.recv() {
                Ok(blocks) => blocks,
                Err(_) => {
                    eprintln!("Error: failed to receive blocks.");
                    return;
                }
            };

            // Look for a genesis block in the received_blocks
            let Some(genesis) = received_blocks
                .iter()
                .find(|b| b.parent_hash.is_empty())
                .cloned()
            else {
                println!("No genesis block found. Nothing to display.");
                return;
            };

            // Create the local blockchain from the received_blocks and the genesis block
            let (blockchain, _remaining_blocks) =
                Blockchain::new_from_genesis_and_vec(genesis, received_blocks);

            println!("Current blockchain state:\n{}", blockchain);
        }

        None => {}
    }

    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn create_test_block(parent_hash: &[u8], nonce_init: u64, miner: &str) -> Block {
            Block::new(
                parent_hash.to_vec(),
                miner.to_string(),
                nonce_init,
                DanceMove::Y,
            )
        }

        #[test]
        fn test_empty_blocks() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let (blockchain, _) =
                Blockchain::new_from_genesis_and_vec(genesis, vec![]);

            assert_eq!(blockchain.blocks.children().len(), 0);
        }

        #[test]
        fn test_single_valid_block() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let genesis_hash = genesis.hash_block().to_vec();

            let block1 = create_test_block(&genesis_hash, 42, "miner1");
            // let mut blockids = BlockHashSet::default();
            let (blockchain, _) =
                Blockchain::new_from_genesis_and_vec(genesis, vec![block1]);
            // assert_eq!(blockids.len(), 1);

            let root = &blockchain.blocks;
            assert_eq!(root.children().len(), 1);
            assert_eq!(root.children()[0].value().miner, "miner1");
        }

        #[test]
        fn test_multiple_levels() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let genesis_hash = genesis.hash_block().to_vec();

            let block1 = create_test_block(&genesis_hash, 42, "miner1");
            let block1_hash = block1.hash_block().to_vec();

            let block2 = create_test_block(&genesis_hash, 43, "miner2");
            let block3 = create_test_block(&block1_hash, 44, "miner3");

            // let mut blockids = BlockHashSet::default();
            let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3],
            );

            //assert_eq!(blockids.len(), 3);

            let root = &blockchain.blocks;
            assert_eq!(root.children().len(), 2); // block1 and block2

            // Find block1 in children
            let block1_node = root
                .children()
                .iter()
                .find(|n| n.value().miner == "miner1")
                .unwrap();

            assert_eq!(block1_node.children().len(), 1); // block3
            assert_eq!(block1_node.children()[0].value().miner, "miner3");
            assert!(remaining.is_empty());

        }

        #[test]
        fn test_orphaned_blocks() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let fake_hash = vec![0xFF; 32]; // Invalid parent hash

            let valid_block = create_test_block(&genesis.hash_block().to_vec(), 42, "miner1");
            let orphan_block = create_test_block(&fake_hash, 10, "miner2");

            let (blockchain, _) = Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![valid_block, orphan_block],
            );

            // Only valid_block should be added
            assert_eq!(blockchain.blocks.children().len(), 1);
            assert_eq!(blockchain.blocks.children()[0].value().miner, "miner1");
        }

        #[test]
        fn test_duplicate_valid_blocks() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let genesis_hash = genesis.hash_block().to_vec();

            let block1 = create_test_block(&genesis_hash, 42, "miner1");
            let block1_hash = block1.hash_block().to_vec();

            let block2 = create_test_block(&genesis_hash, 43, "miner2");
            let block3 = create_test_block(&block1_hash, 43, "miner3");

            //let mut blockids = BlockHashSet::default();

            let (blockchain, _) = Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3],
            );

            //assert_eq!(blockids.len(), 2);

            let root = &blockchain.blocks;
            assert_eq!(root.children().len(), 2); // block1 and block2

            // Find block1 in children
            let block1_node = root
                .children()
                .iter()
                .find(|n| n.value().miner == "miner1")
                .unwrap();

            assert_eq!(block1_node.children().len(), 0); // block3 not added
        }

        #[test]
        fn test_complex_structure() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let genesis_hash = genesis.hash_block().to_vec();

            // Create blocks
            let block1 = create_test_block(&genesis_hash, 42, "miner1");
            let block1_hash = block1.hash_block().to_vec();

            let block2 = create_test_block(&genesis_hash, 43, "miner2");
            let block2_hash = block2.hash_block().to_vec();

            let block3 = create_test_block(&block1_hash, 44, "miner3");
            let block4 = create_test_block(&block2_hash, 45, "miner4");
            let block5 = create_test_block(&block2_hash, 46, "miner5");

            let (blockchain, _) = Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3, block4, block5],
            );

            // Verify structure
            let root = &blockchain.blocks;
            assert_eq!(root.children().len(), 2);

            let block1_node = root
                .children()
                .iter()
                .find(|n| n.value().miner == "miner1")
                .unwrap();
            assert_eq!(block1_node.children().len(), 1);
            assert_eq!(block1_node.children()[0].value().miner, "miner3");

            let block2_node = root
                .children()
                .iter()
                .find(|n| n.value().miner == "miner2")
                .unwrap();
            assert_eq!(block2_node.children().len(), 2);
            assert!(block2_node
                .children()
                .iter()
                .any(|n| n.value().miner == "miner4"));
            assert!(block2_node
                .children()
                .iter()
                .any(|n| n.value().miner == "miner5"));
        }

        #[test]
        fn test_multiple_genesis() {
            let genesis = create_test_block(&[], 0, "Genesis");
            let genesis2 = create_test_block(&[], 42, "Genesis");

            let genesis_hash = genesis.hash_block().to_vec();
            let genesis2_hash = genesis2.hash_block().to_vec();

            let block1 = create_test_block(&genesis_hash, 42, "miner1");
            let block1_hash = block1.hash_block().to_vec();

            let block2 = create_test_block(&genesis_hash, 43, "miner2");
            let block3 = create_test_block(&block1_hash, 44, "miner3");

            let block4 = create_test_block(&genesis2_hash, 42, "miner1");

            let (_, remaining) = Blockchain::new_from_genesis_and_vec(
                genesis,
                vec![block1, block2, block3, block4],
            );

            assert_eq!(remaining.len(), 1);
        }
    }

    mod block;
    mod network;
    mod simpletree;
