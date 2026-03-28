# TeachLink Architecture

This document describes the system architecture, component interactions, and data flows for the TeachLink decentralized knowledge-sharing platform built on the Stellar network using Soroban smart contracts.

---

## Table of Contents

- [High-Level System Architecture](#high-level-system-architecture)
- [Contract Module Map](#contract-module-map)
- [Component Interactions](#component-interactions)
- [Data Flow Diagrams](#data-flow-diagrams)
  - [Cross-Chain Bridge Flow](#cross-chain-bridge-flow)
  - [BFT Consensus Flow](#bft-consensus-flow)
  - [Rewards System Flow](#rewards-system-flow)
  - [Escrow Lifecycle Flow](#escrow-lifecycle-flow)
  - [Content Tokenization Flow](#content-tokenization-flow)
  - [Reputation & Credit Scoring Flow](#reputation--credit-scoring-flow)
- [Storage Model](#storage-model)

---

## High-Level System Architecture

This diagram shows the layers of the TeachLink platform from user-facing clients down to the Stellar network.

```mermaid
graph TD
    subgraph Clients["Client Layer"]
        WEB[Web App]
        MOB[Mobile App]
        EXT[External dApps]
    end

    subgraph Indexer["Indexer / API Layer (TypeScript / NestJS)"]
        IDX[Event Indexer]
        API[REST API]
        DB[(Database)]
        IDX --> DB
        API --> DB
    end

    subgraph Contract["TeachLink Smart Contract (Soroban / Rust)"]
        direction TB
        BRIDGE[Bridge Module]
        REWARDS[Rewards Module]
        ESCROW[Escrow Module]
        TOKEN[Tokenization Module]
        REP[Reputation Module]
        GOV[Governance ⟨planned⟩]
    end

    subgraph Stellar["Stellar Network"]
        LEDGER[Ledger]
        SAC[Stellar Asset Contract]
    end

    subgraph External["External Blockchains"]
        ETH[Ethereum]
        BSC[BNB Chain]
        OTHER[Other EVM Chains]
    end

    WEB --> API
    MOB --> API
    EXT --> API
    API --> Contract
    WEB --> Contract
    MOB --> Contract
    IDX -->|"Listens for events"| Stellar
    Contract --> LEDGER
    Contract --> SAC
    BRIDGE <-->|"Cross-chain messages via validators"| External
```

---

## Contract Module Map

All modules live inside the single `TeachLinkBridge` contract. They are organized into four functional groups.

```mermaid
graph LR
    subgraph Core["Bridge & Consensus"]
        B[bridge]
        BFT[bft_consensus]
        SL[slashing]
        MC[multichain]
        LQ[liquidity]
        MP[message_passing]
        AS[atomic_swap]
    end

    subgraph Platform["Platform Features"]
        RW[rewards]
        ES[escrow]
        TK[tokenization]
        RP[reputation]
        SC[score]
        AR[arbitration]
        ASSESS[assessment]
        SOC[social_learning]
        NOT[notification]
        MOB2[mobile_platform]
    end

    subgraph Ops["Operations & Compliance"]
        EM[emergency]
        AU[audit]
        AN[analytics]
        REP2[reporting]
        PERF[performance]
        BAK[backup]
        ESA[escrow_analytics]
    end

    subgraph Infra["Infrastructure"]
        TY[types]
        ST[storage]
        ER[errors]
        VA[validation]
        EV[events]
    end

    Core --> Infra
    Platform --> Infra
    Ops --> Infra
```

---

## Component Interactions

This diagram shows how the major modules call each other at runtime.

```mermaid
graph TD
    LIB["lib.rs (TeachLinkBridge — contract entry points)"]

    LIB --> B[bridge]
    LIB --> BFT[bft_consensus]
    LIB --> RW[rewards]
    LIB --> ES[escrow]
    LIB --> TK[tokenization]
    LIB --> RP[reputation]
    LIB --> EM[emergency]
    LIB --> AN[analytics]

    B --> BFT
    B --> SL[slashing]
    B --> MC[multichain]
    B --> LQ[liquidity]
    B --> MP[message_passing]
    B --> VA[validation]
    B --> AU[audit]

    ES --> AR[arbitration]
    ES --> ESA[escrow_analytics]
    ES --> VA
    ES --> AU

    BFT --> SL
    BFT --> VA

    RP --> SC[score]
    RP --> AU

    TK --> AU
    TK --> VA

    AN --> PERF[performance]
    AN --> REP2[reporting]

    EM --> AU

    RW --> VA
    RW --> AU

    subgraph shared["Shared Infrastructure"]
        VA[validation]
        AU[audit]
        EV[events]
        ST[storage]
        TY[types]
        ER[errors]
    end
```

---

## Data Flow Diagrams

### Cross-Chain Bridge Flow

Describes the full lifecycle of a cross-chain token transfer from Stellar to an external chain and back.

```mermaid
sequenceDiagram
    actor User
    participant Contract as TeachLink Contract
    participant Validators
    participant BFT as BFT Consensus
    participant Token as Stellar Token (SAC)
    participant ExtChain as External Chain

    Note over User,ExtChain: Outbound (Stellar → External Chain)

    User->>Contract: bridge_out(from, amount, dest_chain, dest_address)
    Contract->>Contract: validation::BridgeValidator::validate_bridge_out()
    Contract->>Token: transfer(user → contract, amount)
    Contract->>Contract: Store BridgeTransaction (status: Pending)
    Contract->>Contract: Emit BridgeInitiatedEvent
    Contract-->>User: nonce

    Validators->>ExtChain: Observe lock event, mint tokens on dest chain

    Note over User,ExtChain: Inbound (External Chain → Stellar)

    ExtChain->>Validators: Burn/lock tokens on external chain
    Validators->>BFT: submit_bridge_vote(proposal_id, validator)
    BFT->>BFT: Check quorum (≥ min_validators votes)
    BFT-->>Contract: Proposal Approved

    Validators->>Contract: complete_bridge(message, validator_signatures)
    Contract->>BFT: Verify signatures meet threshold
    Contract->>Token: transfer(contract → recipient, amount)
    Contract->>Contract: Mark BridgeTransaction complete
    Contract->>Contract: Emit BridgeCompletedEvent
    Contract-->>Validators: Ok
```

---

### BFT Consensus Flow

Shows how validators reach Byzantine Fault Tolerant consensus before a bridge proposal is executed.

```mermaid
flowchart TD
    A([Validator submits vote]) --> B{Proposal exists?}
    B -- No --> C[Create BridgeProposal\nstatus = Pending]
    B -- Yes --> D{Already voted?}
    C --> D
    D -- Yes --> E([Reject: AlreadyVoted])
    D -- No --> F[Record vote\nvote_count += 1]
    F --> G{vote_count ≥ required_votes?}
    G -- No --> H([Wait for more validators])
    G -- Yes --> I[Set status = Approved]
    I --> J[Execute bridge transaction]
    J --> K[Set status = Executed]
    K --> L([Emit event, update SlashingRecord])

    style E fill:#f66,color:#fff
    style H fill:#fa0,color:#fff
    style L fill:#6a6,color:#fff
```

---

### Rewards System Flow

Describes how the reward pool is funded and how rewards are issued and claimed by users.

```mermaid
sequenceDiagram
    actor Admin
    actor Funder
    actor User
    participant Contract as TeachLink Contract
    participant Token as Stellar Token (SAC)

    Note over Funder,Token: Fund the reward pool

    Funder->>Contract: fund_reward_pool(funder, amount)
    Contract->>Token: transfer(funder → contract, amount)
    Contract->>Contract: REWARD_POOL += amount
    Contract->>Contract: Emit RewardPoolFundedEvent

    Note over Admin,User: Issue a reward

    Admin->>Contract: issue_reward(user, amount, reward_type)
    Contract->>Contract: validation::RewardsValidator::validate_issue_reward()
    Contract->>Contract: Check REWARD_POOL ≥ amount
    Contract->>Contract: Update USER_REWARDS[user]
    Contract->>Contract: REWARD_POOL -= amount
    Contract->>Contract: TOTAL_REWARDS_ISSUED += amount
    Contract->>Contract: Emit RewardIssuedEvent

    Note over User,Token: User claims reward

    User->>Contract: claim_reward(user)
    Contract->>Contract: Load USER_REWARDS[user]
    Contract->>Token: transfer(contract → user, pending_amount)
    Contract->>Contract: Mark reward as claimed
    Contract->>Contract: Emit RewardClaimedEvent
```

---

### Escrow Lifecycle Flow

Shows all paths through a multi-signature escrow: normal release, timeout refund, and dispute resolution.

```mermaid
stateDiagram-v2
    [*] --> Pending : create_escrow()\ndepositor transfers tokens to contract

    Pending --> Active : Signers approve\n(threshold met)
    Pending --> Refunded : refund_time elapsed\ncancel_escrow()

    Active --> Released : release_escrow()\nbeneficiary receives funds
    Active --> Disputed : dispute_escrow(reason)\narbitrator assigned

    Disputed --> Resolved_Beneficiary : arbitrator rules for beneficiary\nfunds transferred to beneficiary
    Disputed --> Resolved_Depositor : arbitrator rules for depositor\nfunds returned to depositor

    Released --> [*]
    Refunded --> [*]
    Resolved_Beneficiary --> [*]
    Resolved_Depositor --> [*]
```

---

### Content Tokenization Flow

Describes how educational content is minted as an NFT and transferred with full provenance tracking.

```mermaid
sequenceDiagram
    actor Creator
    actor Buyer
    participant Contract as TeachLink Contract
    participant Audit as Audit Module

    Note over Creator,Audit: Minting a content token

    Creator->>Contract: mint_content_token(creator, metadata)
    Contract->>Contract: validation: validate metadata fields
    Contract->>Contract: Generate token_id
    Contract->>Contract: Store ContentToken\n(owner = creator)
    Contract->>Contract: Create initial ProvenanceRecord
    Contract->>Audit: log_action(MintToken)
    Contract-->>Creator: token_id

    Note over Creator,Buyer: Transferring ownership

    Creator->>Contract: transfer_token(from, to, token_id)
    Creator->>Contract: [require_auth from creator]
    Contract->>Contract: Verify from == current owner
    Contract->>Contract: Update ContentToken.owner = to
    Contract->>Contract: Append ProvenanceRecord\n(from → to, timestamp)
    Contract->>Audit: log_action(TransferToken)
    Contract-->>Buyer: Ok
```

---

### Reputation & Credit Scoring Flow

Shows how user activity is tracked and converted into a reputation score and credit score.

```mermaid
flowchart LR
    subgraph Activities["User Activities"]
        A1[Complete course]
        A2[Submit assessment]
        A3[Bridge transaction]
        A4[Create content token]
        A5[Participate in escrow]
    end

    subgraph Reputation["reputation.rs"]
        R1[update_reputation\nactivity_type, amount]
        R2[UserReputation\n• participation_score\n• completion_rate\n• contribution_quality\n• last_activity]
    end

    subgraph Score["score.rs"]
        S1[calculate_credit_score\nuser]
        S2[CreditScore\n• total_score\n• activity_breakdown\n• risk_level]
    end

    subgraph Audit["audit.rs"]
        AU1[AuditRecord]
    end

    Activities --> R1
    R1 --> R2
    R2 --> S1
    S1 --> S2
    R1 --> AU1

    S2 -->|"Used by"| ES[Escrow — determine trust threshold]
    S2 -->|"Used by"| BFT[BFT — validator weight]
    S2 -->|"Used by"| RW[Rewards — bonus multipliers]
```

---

## Storage Model

The contract uses Soroban's instance and persistent storage. Key storage entries are defined in `storage.rs`.

```mermaid
erDiagram
    CONTRACT_INSTANCE {
        Address TOKEN
        Address ADMIN
        Address FEE_RECIPIENT
        Address REWARDS_ADMIN
        u32     MIN_VALIDATORS
        i128    BRIDGE_FEE
        i128    REWARD_POOL
        i128    TOTAL_REWARDS_ISSUED
        u64     NONCE
        u64     ESCROW_COUNT
    }

    BRIDGE_TXS {
        u64     nonce PK
        Address from
        i128    amount
        u32     destination_chain
        Bytes   destination_address
        String  status
        u64     timestamp
    }

    VALIDATORS {
        Address validator_address PK
        bool    is_active
    }

    SUPPORTED_CHAINS {
        u32  chain_id PK
        bool is_active
    }

    ESCROWS {
        u64     escrow_id PK
        Address depositor
        Address beneficiary
        Address token
        i128    amount
        u32     threshold
        String  status
    }

    USER_REWARDS {
        Address user PK
        i128    pending_amount
        i128    total_claimed
        String  reward_type
        u64     issued_at
    }

    CONTENT_TOKENS {
        u64     token_id PK
        Address owner
        Address creator
        Bytes   metadata_hash
        u64     minted_at
    }

    PROVENANCE_RECORDS {
        u64     token_id FK
        Address from
        Address to
        u64     timestamp
    }

    USER_REPUTATION {
        Address user PK
        u32     participation_score
        u32     completion_rate
        u32     contribution_quality
        u64     last_activity
    }

    CONTRACT_INSTANCE ||--o{ BRIDGE_TXS : "tracks"
    CONTRACT_INSTANCE ||--o{ VALIDATORS : "manages"
    CONTRACT_INSTANCE ||--o{ SUPPORTED_CHAINS : "supports"
    CONTRACT_INSTANCE ||--o{ ESCROWS : "holds"
    CONTRACT_INSTANCE ||--o{ USER_REWARDS : "distributes"
    CONTRACT_INSTANCE ||--o{ CONTENT_TOKENS : "mints"
    CONTENT_TOKENS ||--o{ PROVENANCE_RECORDS : "has history"
    CONTRACT_INSTANCE ||--o{ USER_REPUTATION : "tracks"
```

---

## Keeping Diagrams Updated

When making changes to the contract, update the relevant diagram(s) in this file:

| Change type | Diagram to update |
|---|---|
| New module added | Contract Module Map, Component Interactions |
| New bridge flow or validator logic | Cross-Chain Bridge Flow, BFT Consensus Flow |
| Escrow state change | Escrow Lifecycle Flow |
| New reward type | Rewards System Flow |
| New token operation | Content Tokenization Flow |
| New reputation activity | Reputation & Credit Scoring Flow |
| New storage key added | Storage Model |

All diagrams are written in [Mermaid](https://mermaid.js.org/) and render natively on GitHub.
