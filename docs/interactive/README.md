# Interactive Documentation for TeachLink Contract

This directory contains an interactive documentation system for the TeachLink Stellar smart contract.

## Features

- **Interactive Code Playground**: Run Stellar contract code live in the browser.
- **API Explorer**: Browse and execute contract functions with real-time results.
- **Architecture Visualization**: Visual diagrams of the contract structure.
- **Guided Tutorials**: Step-by-step guides for implementing core features.

## Running the Documentation

1. Ensure you have Rust and Soroban CLI installed.
2. Build the documentation server:
   ```
   cd docs/interactive
   cargo build --release
   ```
3. Run the server:
   ```
   cargo run
   ```
4. Open http://localhost:3000 in your browser.

## Usage

- Click "Deploy Contract Locally" to deploy the contract to a local Stellar network.
- Use the API Explorer to call contract functions.
- Follow the tutorials for guided learning.

## Architecture

The system consists of:

- Backend (Rust/Axum): Serves static files and handles contract invocations via Soroban CLI.
- Frontend (HTML/JS): Interactive UI for exploring the contract.
- Contract Integration: Live execution using the existing deployment scripts.
