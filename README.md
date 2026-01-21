# TeachLink: Decentralized Knowledge-Sharing on Stellar 🌌

TeachLink is a decentralized knowledge-sharing platform powered by **Rust smart contracts** and deployed on the **Stellar blockchain**. It enables tokenized learning rewards, proof-of-participation, and educator incentives—creating a transparent, fair, and scalable system for global education.  

Learners earn tokens for completing courses, quizzes, or tutorials, while creators and educators are incentivized for sharing knowledge. By leveraging Stellar’s ecosystem, TeachLink ensures that rewards are fast, affordable, and accessible worldwide.  

---

## 🚀 Project Goals

- **On-chain user rewards & proof of skill acquisition**  
- **Token logic for incentivizing creators and learners**  
- **Modular, testable Rust-based infrastructure**  

---

## 🌌 Why Stellar Matters for TeachLink

- ⚡ **Low-cost transactions** → Micro-rewards are practical and accessible worldwide.  
- 🚀 **Fast settlement** → Rewards and incentives are distributed instantly.  
- 🌍 **Global reach & interoperability** → Anchors and cross-border rails integrate TeachLink tokens with fiat and assets.  
- 📚 **Focus on inclusion** → Stellar’s mission of democratizing finance aligns with TeachLink’s vision of democratizing knowledge.  

---

## 🛠️ Tech Stack

- **Rust** → Smart contract development  
- **Soroban (Stellar)** → Smart contract platform  
- **Stellar Network** → Fast, low-cost, global blockchain infrastructure  
- **Custom Indexer (NestJS + Horizon API + Soroban RPC)** → Real-time contract and transaction monitoring  

---

## 🏗️ Architecture Overview

```
─────────────────────────────────────────────────────────────┐    │
│  │              SOROBAN SMART CONTRACTS (Rust)                   │    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │                                                               │    │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │    │
│  │  │ Loan Contract  │  │Insurance Pool  │  │ Escrow Manager │ │    │
│  │  │ - Origination  │  │ - Premium Pool │  │ - Multi-sig    │ │    │
│  │  │ - Disbursement │  │ - Claims       │  │ - Conditions   │ │    │
│  │  │ - Repayment    │  │ - Oracles      │  │ - Release      │ │    │
│  │  │ - Collateral   │  │ - Payouts      │  │ - Disputes     │ │    │
│  │  └────────────────┘  └────────────────┘  └────────────────┘ │    │
│  │                                                               │    │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │    │
│  │  │Supply Chain    │  │Credit Scoring  │  │ Treasury       │ │    │
│  │  │ - Tokenization │  │ - History      │  │ - Multi-sig    │ │    │
│  │  │ - Provenance   │  │ - Reputation   │  │ - Governance   │ │    │
│  │  │ - Transfer     │  │ - Verification │  │ - Withdrawals  │ │    │
│  │  └────────────────┘  └────────────────┘  └────────────────┘ │    │
│  │                                                               │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │                    STELLAR CORE                               │    │
│  │  - Payment Operations  - DEX Operations                       │    │
│  │  - Account Management  - Asset Issuance                       │    │
│  │  - Trust Lines        - Clawback                              │    │
│  └──────────────────────────────────────────────────────────────┘ 

---

## 📖 Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/rinafcode/teachLink_contract.git
cd teachLink_contract
```

### 2. Build and Test Contracts
```bash
cargo build
cargo test
```

### 3. Environment Setp
```bash
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
DEPLOYER_SECRET_KEY=S[YOUR_SECRET_KEY]
```

### 4. Deploy Contract
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/teachlink_contract.wasm \
  --source deployer \
  --network testnet
```

📦#### Installation
## Dependencies
-- Rust Toolchain (with wasm32-unknown-unknown target)

-- Stellar CLI (cargo install --locked stellar-cli --features opt)

-- pnpm / npm / yarn for JS services

-- Terraform & Helm for infra setup

### Testing
cargo test
cargo tarpaulin --out Html
./scripts/test-contracts-testnet.sh

### Load Testing
k6 run load-tests/contract-invocations.js

🤝 Contributing
We welcome contributions from developers, educators, and blockchain enthusiasts!

Fork the repo

Create a feature branch

Submit a pull request

📜 License
This project is licensed under the MIT License. See [Looks like the result wasn't safe to show. Let's switch things up and try something else!] for details.

✨ In short, Stellar isn’t just the blockchain TeachLink runs on—it’s the foundation that makes decentralized education rewards scalable, affordable, and globally relevant.



