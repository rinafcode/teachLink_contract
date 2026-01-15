TeachLink Cairo Smart Contracts TeachLink is a decentralized knowledge-sharing
platform. This repository contains Rust smart contracts to support core
on-chain features such as tokenized learning rewards, proof-of-participation,
and educator incentives—all deployed on Stellar.

🚀 Project Goals Enable on-chain user rewards and proof of skill acquisition.

Implement token logic for incentivizing creators and learners.

Build a modular, testable Rust-based infrastructure compatible with Stellar.

🛠️ Getting Started

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
## 🛠️ Technology Stack
#### **Blockchain Indexer**

{
  "framework": "Custom NestJS service",
  "stellar-integration": [
    "Horizon API (streaming)",
    "Stellar SDK (transaction parsing)",
    "Soroban RPC (contract events)"
  ],
  "processing": [
    "Bull queue for async processing",
    "Worker threads for parallel indexing",
    "Checkpointing for recovery"
  ],
  "data-sync": "Real-time event streaming + batch catchup"
}
```

**Indexed Data**:
- All payment transactions
- Smart contract invocations
- Asset transfers and trust lines
- Account creations and updates
- DEX trades and liquidity changes

### Prerequisites

Ensure you have the following installed on your system:

```bash
# Required
- Stellar CLI (stellar-cli)

# Optional but recommended
- pnpm >= 8.x (faster than npm)
- Terraform >= 1.6
- Helm >= 3.12
```

---

## 📦 Installation

### 1. Clone the Repository

```bash
git clone https://github.com/stellartech/agrichain-finance.git
cd agrichain-finance
```

### 2. Install Dependencies

#### Install All Project Dependencies
```bash
# Install root dependencies and workspace packages
pnpm install

# Or using npm
npm install

# Or using yarn
yarn install
```

This will install dependencies for:
- Smart contracts (Rust/Soroban)
- Shared libraries

#### Install Rust Toolchain for Smart Contracts
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Verify installation
stellar --version
```

#### Install Additional Tools
```bash
# Install Soroban smart contract dependencies
cd contracts
cargo build

## ⚙️ Setup

### 1. Environment Configuration
# Stellar Network
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
PLATFORM_STELLAR_SECRET=S[YOUR_SECRET_KEY_HERE]

#### Smart Contract Configuration
```bash
cp contracts/.env.example contracts/.env
nano contracts/.env
```

**contracts/.env**
```env
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
DEPLOYER_SECRET_KEY=S[YOUR_SECRET_KEY]
```

---

### 4. Deploy Smart Contracts

#### Build Contracts
```bash
cd contracts

# Build all contracts
stellar contract build

#### Deploy to Testnet
```bash
# Deploy Advanced Subscription  contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/Advanced_Subscription.wasm \
  --source deployer \
  --network testnet

# Save the contract ID returned (starts with C)
# Example: CCXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Deploy other contracts similarly
./scripts/deploy-all-contracts.sh

#### Initialize Contracts
```bash
# Initialize su
stellar contract invoke \
  --id C[ADVANCE_SUBSCRIPTION_CONTRACT_ID] \
  --source admin \
  --network testnet \
  -- \
  initialize \
  --admin G[ADMIN_PUBLIC_KEY]

# Follow similar pattern for other contracts
```

---

**Blockchain Indexer**

### Smart Contract Tests

```bash
cd contracts

# Run all contract tests
cargo test

# Test specific contract
cargo test --package Advanced Subscription

# Test with coverage
cargo tarpaulin --out Html

# Integration tests on testnet
./scripts/test-contracts-testnet.sh
```

### Load Testing
# Smart contract load test
k6 run load-tests/contract-invocations.js


# Smart contracts
stellar network add \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015" \
  testnet
```

### Account Funding

```bash
# Testnet: Use Friendbot
curl "https://friendbot.stellar.org?addr=G[YOUR_PUBLIC_KEY]"

# Or using Stellar CLI
stellar keys fund alice --network testnet

# Pubnet: Purchase XLM from exchanges
# Minimum balance: 1 XLM (base reserve) Indexer Database**
```yaml
Type: PostgreSQL (separate instance)
Purpose: Mirror Stellar blockchain data locally
Tables:
  - Stellar accounts
  - Transaction history
  - Smart contract events
  - Asset movements
Sync: Real-time via Horizon streaming
```

---

### Blockchain Layer

#### **Smart Contracts (Rust + Soroban)**

**1. Access Control Contract**
```rust
// contracts/access-control/src/lib.rs
pub struct AccessControl {
    admin: Address,
    access_terms: Map<Symbol, LoanTerms>,
    active_access: Map<Address, LoanDetails>,
    access_schedule: Map<Symbol, Vec<Repayment>>
}


**2. Parametric Insurance Contract**
```rust
// contracts/insurance-pool/src/lib.rs
pub struct InsurancePool {
    premium_pool: Address,
    oracle_address: Address,
    policies: Map<Symbol, PolicyDetails>,
    claims: Map<Symbol, ClaimStatus>
}


**3. Course Contract**
```rust
// contracts/supply-chain/src/lib.rs
pub struct CourseToken {
    batch_id: Symbol,
    origin_farm: Address,
    certifications: Vec<Certification>,
    custody_chain: Vec<Transfer>,
    metadata_uri: String // IPFS hash
}


**4. Escrow Contract**
```rust
// contracts/escrow/src/lib.rs
pub struct Escrow {
    teacher: Address,
    student: Address,
    arbiter: Address,
    amount: i128,
    conditions: Vec<Condition>,
    status: EscrowStatus
}


**5. Grade Scoring Contract**
```rust
// contracts/grade-score/src/lib.rs
pub struct GradeScore {
    student: Address,
    score: u32,
    factors: CreditFactors,
    history: Vec<CreditEvent>,
    last_updated: u64
}


**Smart Contract Development Tools**
```bash
stellar-cli          # Soroban CLI tools
soroban-sdk         # Rust SDK for contract development
cargo-make          # Build automation
wasm-opt            # WASM optimization
soroban-test        # Contract testing framework
```

---

