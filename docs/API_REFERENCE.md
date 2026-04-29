# TeachLink Smart Contracts - API Reference

Complete API documentation for the TeachLink decentralized knowledge-sharing platform smart contracts.

---

## Quick Reference

| Contract | Purpose | Key Functions |
|----------|---------|---------------|
| **TeachLink** | Core platform contract | Bridge, Rewards, Escrow, Tokenization |
| **Governance** | On-chain governance | Proposals, Voting, Execution |
| **Insurance** | Course completion protection | Premiums, Claims, Payouts |

---

## TeachLink Contract

The main contract combining all platform features.

### Initialization

```rust
/// Initialize the bridge component
TeachLinkBridge::initialize(
    env: Env,
    token: Address,           // Token for bridging
    admin: Address,           // Admin address
    min_validators: u32,      // Minimum validator signatures
    fee_recipient: Address,   // Fee collection address
)

/// Initialize the rewards system
TeachLinkBridge::initialize_rewards(
    env: Env,
    token: Address,           // Reward token
    rewards_admin: Address,   // Rewards administrator
)
```

### Cross-Chain Bridge

Bridge tokens between Stellar and other blockchains.

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `bridge_out` | Lock tokens for bridging to another chain | User |
| `complete_bridge` | Complete incoming bridge with validator signatures | Validators |
| `cancel_bridge` | Cancel and refund a bridge transaction | - |
| `add_validator` | Add a validator address | Admin |
| `remove_validator` | Remove a validator address | Admin |
| `add_supported_chain` | Enable a destination chain | Admin |
| `set_bridge_fee` | Update bridge fee | Admin |

```rust
// Bridge tokens out
let nonce = TeachLinkBridge::bridge_out(
    env,
    from: Address,
    amount: i128,
    destination_chain: u32,
    destination_address: Bytes,
) -> u64;

// Complete incoming bridge
TeachLinkBridge::complete_bridge(
    env,
    message: CrossChainMessage,
    validator_signatures: Vec<Address>,
);
```

### Token Rewards

Incentivize learning with token rewards.

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `fund_reward_pool` | Add tokens to reward pool | Funder |
| `issue_reward` | Issue reward to user | Admin |
| `claim_rewards` | Claim pending rewards | User |
| `set_reward_rate` | Configure reward rates | Admin |
| `get_user_rewards` | Get user's reward info | - |
| `get_reward_pool_balance` | Get pool balance | - |

```rust
// Fund the reward pool
TeachLinkBridge::fund_reward_pool(env, funder: Address, amount: i128);

// Issue a reward
TeachLinkBridge::issue_reward(
    env,
    recipient: Address,
    amount: i128,
    reward_type: String,  // e.g., "course_completion"
);

// Claim rewards
TeachLinkBridge::claim_rewards(env, user: Address);
```

### Multi-Signature Escrow

Secure payments with configurable release conditions.

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `create_escrow` | Create new escrow | Depositor |
| `approve_escrow_release` | Sign for release | Signer |
| `release_escrow` | Release funds to beneficiary | Caller |
| `refund_escrow` | Refund to depositor | Depositor |
| `cancel_escrow` | Cancel before approvals | Depositor |
| `dispute_escrow` | Raise a dispute | Party |
| `resolve_escrow` | Resolve dispute | Arbitrator |

```rust
// Create escrow
let escrow_id = TeachLinkBridge::create_escrow(
    env,
    depositor: Address,
    beneficiary: Address,
    token: Address,
    amount: i128,
    signers: Vec<Address>,    // Authorized signers
    threshold: u32,           // Required signatures
    release_time: Option<u64>,
    refund_time: Option<u64>,
    arbitrator: Address,
) -> u64;

// Approve release (multi-sig)
let approval_count = TeachLinkBridge::approve_escrow_release(
    env, escrow_id: u64, signer: Address
) -> u32;

// Release when conditions met
TeachLinkBridge::release_escrow(env, escrow_id: u64, caller: Address);
```

### Content Tokenization

Mint NFTs for educational content.

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `mint_content_token` | Create new content NFT | Creator |
| `transfer_content_token` | Transfer ownership | Owner |
| `update_content_metadata` | Update title/description/tags | Owner |
| `set_content_token_transferable` | Toggle transferability | Owner |
| `get_content_token` | Get token details | - |
| `get_content_provenance` | Get ownership history | - |

```rust
// Mint content token
let token_id = TeachLinkBridge::mint_content_token(
    env,
    creator: Address,
    title: Bytes,
    description: Bytes,
    content_type: ContentType,  // Course, Material, Lesson, etc.
    content_hash: Bytes,        // IPFS hash
    license_type: Bytes,
    tags: Vec<Bytes>,
    is_transferable: bool,
    royalty_percentage: u32,    // Basis points (500 = 5%)
) -> u64;

// Transfer token
TeachLinkBridge::transfer_content_token(
    env,
    from: Address,
    to: Address,
    token_id: u64,
    notes: Option<Bytes>,
);
```

### Reputation & Credit Scoring

| Function | Description |
|----------|-------------|
| `update_participation` | Add participation points |
| `update_course_progress` | Record course start/completion |
| `rate_contribution` | Rate a user's contribution |
| `get_user_reputation` | Get full reputation profile |
| `record_course_completion` | Admin: record completion with points |
| `record_contribution` | Admin: record contribution |
| `get_credit_score` | Get user's credit score |

---

## Governance Contract

Decentralized governance through token-weighted voting.

### Proposal Lifecycle

```
1. CREATE → 2. VOTE → 3. FINALIZE → 4. EXECUTE
     ↓          ↓          ↓            ↓
   Active    Active    Passed/Failed  Executed
```

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `initialize` | Set up governance | - |
| `create_proposal` | Create new proposal | Token holder |
| `cast_vote` | Vote on proposal | Token holder |
| `finalize_proposal` | Finalize after voting | - |
| `execute_proposal` | Execute passed proposal | Anyone |
| `cancel_proposal` | Cancel proposal | Proposer/Admin |
| `update_config` | Update parameters | Admin |

```rust
// Create proposal
let proposal_id = Governance::create_proposal(
    env,
    proposer: Address,
    title: Bytes,
    description: Bytes,
    proposal_type: ProposalType,  // FeeChange, ParameterUpdate, etc.
    execution_data: Option<Bytes>,
) -> u64;

// Cast vote
let voting_power = Governance::cast_vote(
    env,
    proposal_id: u64,
    voter: Address,
    direction: VoteDirection,  // For, Against, Abstain
) -> i128;

// Finalize after voting ends
Governance::finalize_proposal(env, proposal_id: u64);

// Execute after delay
Governance::execute_proposal(env, proposal_id: u64, executor: Address);
```

---

## Insurance Contract

Course completion protection insurance.

### Claim Workflow

```
1. PAY_PREMIUM → 2. FILE_CLAIM → 3. PROCESS (Oracle) → 4. PAYOUT
       ↓              ↓               ↓                    ↓
    Insured        Pending      Verified/Rejected         Paid
```

| Function | Description | Auth Required |
|----------|-------------|---------------|
| `initialize` | Set up pool | - |
| `pay_premium` | Purchase insurance | User |
| `file_claim` | File a claim | User (insured) |
| `process_claim` | Verify/reject claim | Oracle |
| `payout` | Pay verified claim | - |
| `withdraw` | Withdraw pool funds | Admin |

```rust
// User pays premium
InsurancePool::pay_premium(env, user: Address);

// User files claim
let claim_id = InsurancePool::file_claim(
    env, user: Address, course_id: u64
) -> u64;

// Oracle processes claim
InsurancePool::process_claim(
    env, claim_id: u64, result: bool  // true = approved
);

// Payout verified claim
InsurancePool::payout(env, claim_id: u64);
```

---

## Data Types

### Bridge Types

| Type | Description |
|------|-------------|
| `BridgeTransaction` | Cross-chain bridge state |
| `CrossChainMessage` | Incoming bridge message |

### Escrow Types

| Type | Description |
|------|-------------|
| `Escrow` | Multi-sig escrow state |
| `EscrowStatus` | Pending, Released, Refunded, Disputed, Cancelled |
| `DisputeOutcome` | ReleaseToBeneficiary, RefundToDepositor |

### Tokenization Types

| Type | Description |
|------|-------------|
| `ContentToken` | Educational content NFT |
| `ContentMetadata` | Title, description, creator, etc. |
| `ContentType` | Course, Material, Lesson, Assessment, Certificate |
| `ProvenanceRecord` | Ownership transfer record |
| `TransferType` | Mint, Transfer, License, Revoke |

### Governance Types

| Type | Description |
|------|-------------|
| `Proposal` | Governance proposal |
| `ProposalType` | FeeChange, ParameterUpdate, FeatureToggle, Custom |
| `ProposalStatus` | Pending, Active, Passed, Failed, Executed, Cancelled |
| `Vote` | Individual vote record |
| `VoteDirection` | For, Against, Abstain |

### User Types

| Type | Description |
|------|-------------|
| `UserReward` | User's reward balance and history |
| `UserReputation` | Participation, completion rate, quality |
| `Contribution` | User contribution record |
| `ContributionType` | Content, Code, Community, Governance |

---

## Error Handling

All contracts use Rust panics for error conditions. Common errors:

| Error | Cause |
|-------|-------|
| "Already initialized" | Contract already set up |
| "Not initialized" | Contract not set up |
| "Insufficient balance" | Not enough tokens |
| "Not authorized" | Missing required authorization |
| "Not found" | Resource doesn't exist |
| "Already processed" | Action already completed |

---

## Building Documentation

Generate Rust documentation:

```bash
cargo doc --no-deps --document-private-items
open target/doc/teachlink_contract/index.html
```
