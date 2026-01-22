<div align="center">

```
                TTTTT eeeee aaaaa ccccc h   h L     i     n   n k   k
                  T   e     a   a c     h   h L           nn  n k  k
                  T   eeee  aaaaa c     hhhhh L     i     n n n kkk
                  T   e     a   a c     h   h L     i     n  nn k  k
                  T   eeeee a   a ccccc h   h LLLLL i     n   n k   k
```

</div>

# TeachLink: Decentralized Knowledge-Sharing on Stellar

TeachLink is a Soroban smart contract that powers tokenized learning rewards on the Stellar network. This repository contains the Rust smart contract and developer tooling for building, testing, and deploying the contract to Stellar testnet or mainnet.

## Table of Contents

- Overview
- Onboarding
- Architecture
- Development Workflow
- Contribution Guidelines
- Troubleshooting
- License

## Overview

TeachLink enables tokenized learning rewards, proof-of-participation, and educator incentives. The contract is written in Rust for Soroban, Stellar's smart contract platform.

## Onboarding

The onboarding flow is designed to take you from clone to first deployment with minimal guesswork.

### 1) Clone the repository

```bash
git clone https://github.com/rinafcode/teachLink_contract.git
cd teachLink_contract
```

### 2) Automated environment setup (dependency validation)

Run the setup script to validate required dependencies and create a local `.env` file if needed:

```bash
./scripts/setup-env.sh
```

What it checks:

- `rustc`, `cargo`, and `rustup`
- `wasm32-unknown-unknown` target
- `stellar` or `soroban` CLI
- local `.env` bootstrap from `.env.example`

### 3) Configure environment variables

Update `.env` with your deployment settings:

```bash
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
DEPLOYER_SECRET_KEY=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

If you do not have a key, generate one with the Stellar CLI:

```bash
stellar keys generate --global teachlink-deployer
```

### 4) Build and test the contract

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
cargo test
```

### 5) Interactive first deployment tutorial

The tutorial script walks you through building, funding, and deploying to testnet:

```bash
./scripts/first-deploy.sh
```

Common options:

```bash
./scripts/first-deploy.sh --network testnet --identity teachlink-deployer
./scripts/first-deploy.sh --skip-build
./scripts/first-deploy.sh --dry-run
```

### 6) Network-specific deployment scripts

Use the network-aware deployment script with managed config files:

```bash
./scripts/deploy.sh --network testnet
./scripts/deploy.sh --network mainnet
./scripts/deploy.sh --network local
```

Convenience wrappers:

```bash
./scripts/deploy-testnet.sh
./scripts/deploy-mainnet.sh
./scripts/deploy-local.sh
```

Configuration files live under `config/networks/` and can be customized per environment:

```bash
config/networks/testnet.env
config/networks/mainnet.env
config/networks/local.env
```

## Architecture

```
Client Apps
    |
    v
Indexer / API Layer (optional)
    |
    v
Soroban Smart Contract (Rust)
    |
    v
Stellar Network
```

Key project paths:

- `contracts/teachlink`: Soroban smart contract source
- `scripts/`: onboarding and deployment script

## Development Workflow

Build the WASM:

```bash
cargo build --release --target wasm32-unknown-unknown -p teachlink-contract
```

Run unit tests:

```bash
cargo test
```

Lint and format:

```bash
cargo fmt
cargo clippy --all-targets --all-features
```

## Contribution Guidelines

We welcome contributions that improve contract quality, developer experience, and documentation.

### How to contribute

1. Fork the repo and create a feature branch.
2. Make focused changes with tests.
3. Run the full test and lint suite.
4. Open a PR with a clear summary and testing notes.

### Code example (contract + test)

When adding contract entrypoints, include unit tests in the same module or under `#[cfg(test)]`:

```rust
#[contractimpl]
impl TeachLinkContract {
    #[must_use]
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }
}

#[test]
fn hello_returns_input() {
    let env = Env::default();
    let input = Symbol::new(&env, "teachlink");
    let out = TeachLinkContract::hello(env.clone(), input);
    assert_eq!(out, Symbol::new(&env, "teachlink"));
}
```

### Testing requirements

- All new contract logic must include unit tests.
- `cargo test` must pass.
- `cargo fmt` and `cargo clippy --all-targets --all-features` must pass with no new warnings.

## Troubleshooting

Use `./scripts/setup-env.sh` as your first diagnostic. It prints missing dependencies and how to fix them.

Common issues:

- `Missing command: stellar or soroban`
  - Install the CLI: `cargo install --locked stellar-cli --features opt`
- `Rust target not installed: wasm32-unknown-unknown`
  - Run: `rustup target add wasm32-unknown-unknown`
- `WASM not found` during deployment
  - Rebuild: `cargo build --release --target wasm32-unknown-unknown -p teachlink-contract`
  - Verify the path: `target/wasm32-unknown-unknown/release/teachlink_contract.wasm`
- `DEPLOYER_SECRET_KEY` is empty
  - Generate a key: `stellar keys generate --global teachlink-deployer`
  - Update `.env` with the secret key
- `Account not funded` or `transaction failed` on testnet
  - Re-run the tutorial without `--skip-fund`
  - Or fund manually: `https://friendbot.stellar.org?addr=<PUBLIC_KEY>`
- `curl not found` while funding
  - Install curl or fund the account manually using the friendbot URL

## License

This project is licensed under the MIT License. See `LICENSE` for details.
