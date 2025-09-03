# Rust Blockchain Project

This project implements a simplified blockchain in Rust, with the following features:
- Block creation and validation with Proof of Work (PoW)
- Randomly chosen dance move stored in each block
- Local server to share blocks between miners
- Mining loop with deterministic parent selection (deepest block, lowest nonce in case of tie)

## Project Structure

```
project2025/
 ├─ Cargo.toml          # Workspace definition
 ├─ miner/              # Miner code
 │   ├─ src/
 │   │   ├─ block.rs       # Block structure, PoW logic
 │   │   ├─ network.rs     # Network communication with server
 │   │   ├─ miner.rs       # Mining logic, CLI
 │   │   ├─ simpletree.rs  # Blockchain tree structure
 │   │   └─ lib.rs
 │   └─ Cargo.toml
 └─ server/             # Local blockchain server
     ├─ src/main.rs
     └─ Cargo.toml
```

## Requirements

- Rust (latest stable version)
- Cargo (comes with Rust)
- Internet connection for downloading dependencies

## Building

At the root of the project, run:

```bash
cargo build
```

This will compile both the `miner` and `server` binaries.

## Running the Server

Start the local blockchain server:

```bash
cargo run --bin server -- -p 8080 -d 10
```

Options:
- `-p PORT` : listening port (default: 8080)
- `-d DIFFICULTY` : proof-of-work difficulty (default: 10)

## Running a Miner

Run a miner that connects to the server:

```bash
cargo run --bin miner mine -m "MinerName" -d 10
```

Options:
- `-m NAME` : miner name
- `-d DIFFICULTY` : proof-of-work difficulty

The miner will:
1. Connect to the server
2. Fetch existing blocks
3. Create or use the genesis block
4. Mine new blocks by solving PoW
5. Send valid blocks to the server

## Main Features

- **Proof of Work**: Blocks must satisfy a difficulty condition on their hash.
- **Random Dance Move**: Each block contains a random dance move (Y, M, C, A).
- **Deterministic Parent Selection**: Always mines on the deepest chain, lowest nonce on tie.
- **Network Synchronization**: Server broadcasts blocks to all miners.

## License

This project is for educational purposes. No specific license applies by default.
