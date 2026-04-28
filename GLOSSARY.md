# TeachLink Project Glossary

This document defines key terms and concepts used across the TeachLink ecosystem, including smart contracts, the indexer runtime, and the recommendation system.

---

## 🏗️ Core Architecture

### Soroban
Stellar's smart contract platform. TeachLink's core logic is implemented as Soroban smart contracts written in Rust.

### Indexer Runtime
A high-performance service that monitors the Stellar blockchain in real-time, processes events emitted by TeachLink contracts, and stores the data in a relational database for fast querying.

### Recommendation System
The intelligence layer of TeachLink that uses machine learning (TensorFlow/PyTorch) to personalize learning paths and suggest relevant content to users.

---

## 📜 Smart Contract Concepts

### Tokenization
The process of representing learning content, certificates, or achievements as digital tokens (NFTs or SFTs) on the blockchain.

### Reputation Scoring
A decentralized mechanism for calculating and storing user trust scores based on their contributions, learning history, and peer assessments.

### Multi-sig Escrow
A security feature where multiple signatures are required to authorize the release of funds or the execution of critical contract operations.

### BFT Consensus
Byzantine Fault Tolerance. Used in the context of decentralized governance or cross-chain bridge validation to ensure agreement even in the presence of malicious actors.

### Slashing
A penalty mechanism where a portion of a participant's staked tokens is confiscated due to malicious behavior or failure to meet protocol obligations.

---

## 🛡️ Insurance & Risk (Insurance Module)

### Parametric Trigger
A predefined condition (e.g., a specific event recorded on-chain) that automatically triggers an insurance claim payout without requiring manual assessment.

### Risk Profile
A data structure that quantifies the risk associated with a user, course, or pool, used to calculate insurance premiums.

### Loss Ratio
The ratio of claims paid out to premiums collected. A key metric for assessing the health of an insurance pool.

### Reinsurance
A mechanism where risk is shared with external partners or other pools to ensure the system can handle large, unexpected claims.

---

## 🌐 Cross-Chain & Integration

### Cross-Chain Bridge
The infrastructure that allows TeachLink to interact with and transfer assets or data to other blockchain networks beyond Stellar.

### Oracle
A service that provides external, real-world data to the smart contracts (e.g., verifying a student's completion of an external course).

---

## 🛠️ Development & Tooling

### WASM (WebAssembly)
The compilation target for Soroban smart contracts. Rust code is compiled to `.wasm` files for deployment on the Stellar network.

### Friendbot
A service on the Stellar Testnet that provides free test tokens (XLM) to developers for testing their applications and contracts.

### Horizon / RPC
The API layers used to interact with the Stellar network. Horizon provides a RESTful API, while Soroban RPC provides specialized methods for smart contract interaction.
