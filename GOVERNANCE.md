# TeachLink Governance Contract

A comprehensive decentralized governance system for the TeachLink platform that
enables token holders to participate in platform decision-making through
proposals, voting, and delegation.

## Overview

The TeachLink Governance contract implements a sophisticated governance system
with the following key features:

- **Proposal Creation & Management**: Token holders can create, vote on, and
  execute proposals
- **Token-Weighted Voting**: Voting power is proportional to token holdings
- **Delegation System**: Users can delegate voting power to trusted
  representatives
- **Execution Mechanism**: Approved proposals can execute on-chain actions
- **Parameter Management**: Governance parameters can be modified through
  successful proposals

## Contract Architecture

### Core Components

1. **Proposal System**: Creates and manages governance proposals
2. **Voting Mechanism**: Handles vote casting with token-weighted power
3. **Delegation Engine**: Manages vote delegation between users
4. **Execution Framework**: Executes approved proposals on-chain
5. **Parameter Manager**: Controls governance parameters through proposals

### Key Data Structures

#### Proposal

```cairo
struct Proposal {
    id: u256,
    title: ByteArray,
    description: ByteArray,
    proposer: ContractAddress,
    target: ContractAddress,
    calldata: Span<felt252>,
    value: u256,
    votes_for: u256,
    votes_against: u256,
    votes_abstain: u256,
    start_time: u64,
    end_time: u64,
    executed: bool,
    canceled: bool,
}
```

#### Vote

```cairo
struct Vote {
    voter: ContractAddress,
    proposal_id: u256,
    support: u8, // 0 = against, 1 = for, 2 = abstain
    weight: u256,
    reason: ByteArray,
}
```

#### Delegation

```cairo
struct Delegation {
    delegator: ContractAddress,
    delegate: ContractAddress,
    timestamp: u64,
}
```

#### GovernanceParameters

```cairo
struct GovernanceParameters {
    voting_delay: u64,       // Delay before voting starts (in seconds)
    voting_period: u64,      // Voting period duration (in seconds)
    proposal_threshold: u256, // Minimum tokens needed to create proposal
    quorum_threshold: u256,   // Minimum participation for valid vote
    execution_delay: u64,     // Delay before execution after approval
}
```

## Proposal States

The governance system defines the following proposal states:

- **PENDING (0)**: Proposal created but voting hasn't started
- **ACTIVE (1)**: Voting is currently active
- **CANCELED (2)**: Proposal was canceled by proposer or admin
- **DEFEATED (3)**: Proposal failed (didn't meet quorum or more against votes)
- **SUCCEEDED (4)**: Proposal passed and is ready for execution
- **QUEUED (5)**: Proposal is queued for execution after delay
- **EXPIRED (6)**: Proposal execution window has expired
- **EXECUTED (7)**: Proposal has been successfully executed

## Key Functions

### Proposal Management

#### `create_proposal`

Creates a new governance proposal.

```cairo
fn create_proposal(
    title: ByteArray,
    description: ByteArray,
    target: ContractAddress,
    calldata: Span<felt252>,
    value: u256
) -> u256
```

**Requirements:**

- Caller must have sufficient tokens (≥ proposal_threshold)
- Contract must be initialized

**Returns:** Proposal ID

#### `cancel_proposal`

Cancels an existing proposal.

```cairo
fn cancel_proposal(proposal_id: u256)
```

**Requirements:**

- Only proposer or contract owner can cancel
- Proposal must be in PENDING or ACTIVE state

#### `execute_proposal`

Executes an approved proposal.

```cairo
fn execute_proposal(proposal_id: u256)
```

**Requirements:**

- Proposal must be in SUCCEEDED state
- Execution delay must have passed

### Voting Functions

#### `cast_vote`

Casts a vote on a proposal.

```cairo
fn cast_vote(proposal_id: u256, support: u8, reason: ByteArray)
```

**Parameters:**

- `support`: 0 = against, 1 = for, 2 = abstain
- `reason`: Explanation for the vote

**Requirements:**

- Proposal must be in ACTIVE state
- Voter must not have already voted
- Voter must have voting power > 0

#### `cast_vote_with_signature`

Casts a vote using a signature (for meta-transactions).

```cairo
fn cast_vote_with_signature(
    proposal_id: u256,
    support: u8,
    reason: ByteArray,
    signature: Span<felt252>
)
```

### Delegation Functions

#### `delegate`

Delegates voting power to another address.

```cairo
fn delegate(delegate: ContractAddress)
```

#### `undelegate`

Removes delegation and returns voting power to self.

```cairo
fn undelegate()
```

#### `delegate_by_signature`

Delegates voting power using a signature.

```cairo
fn delegate_by_signature(
    delegator: ContractAddress,
    delegate: ContractAddress,
    signature: Span<felt252>
)
```

### View Functions

#### `get_proposal`

Returns proposal details.

```cairo
fn get_proposal(proposal_id: u256) -> Proposal
```

#### `get_proposal_state`

Returns current proposal state.

```cairo
fn get_proposal_state(proposal_id: u256) -> u8
```

#### `get_voting_power`

Returns voting power for an account at a specific timestamp.

```cairo
fn get_voting_power(account: ContractAddress, timestamp: u64) -> u256
```

#### `get_delegate`

Returns the delegate for an account.

```cairo
fn get_delegate(account: ContractAddress) -> ContractAddress
```

#### `has_voted`

Checks if an account has voted on a proposal.

```cairo
fn has_voted(proposal_id: u256, voter: ContractAddress) -> bool
```

## Governance Flow

### 1. Proposal Creation

1. Token holder with sufficient tokens creates a proposal
2. Proposal enters PENDING state
3. Voting delay period begins

### 2. Voting Period

1. After voting delay, proposal becomes ACTIVE
2. Token holders cast votes (FOR/AGAINST/ABSTAIN)
3. Votes are weighted by token holdings
4. Delegation is considered for voting power calculation

### 3. Proposal Resolution

1. After voting period ends, proposal state is determined:
   - If quorum not met: DEFEATED
   - If more against than for: DEFEATED
   - If more for than against and quorum met: SUCCEEDED

### 4. Execution

1. SUCCEEDED proposals can be executed after execution delay
2. On-chain actions are performed via `call_contract_syscall`
3. Proposal marked as EXECUTED

## Security Features

- **Proposal Threshold**: Prevents spam by requiring minimum token ownership
- **Quorum Requirements**: Ensures meaningful participation before proposals
  pass
- **Execution Delay**: Provides time buffer before critical changes take effect
- **Authorization Checks**: Only authorized parties can cancel or execute
  proposals
- **Double-Vote Prevention**: Users cannot vote multiple times on same proposal

## Events

The contract emits comprehensive events for transparency:

- `ProposalCreated`: New proposal created
- `ProposalCanceled`: Proposal canceled
- `ProposalExecuted`: Proposal executed
- `VoteCast`: Vote recorded
- `DelegateChanged`: Delegation changed
- `GovernanceParametersUpdated`: Parameters modified

## Integration

### With Token Contract

The governance contract integrates with the TeachLink token contract to:

- Check token balances for voting power
- Verify proposal thresholds
- Calculate delegation weights

### Initialization

```cairo
// Initialize governance contract
let params = GovernanceParameters {
    voting_delay: 86400,      // 1 day
    voting_period: 259200,    // 3 days
    proposal_threshold: 1000, // 1000 tokens
    quorum_threshold: 10000,  // 10000 tokens
    execution_delay: 172800,  // 2 days
};

governance.initialize(token_address, params);
```

### Example Usage

```cairo
// Create a proposal to update platform parameters
let proposal_id = governance.create_proposal(
    "Update Fee Structure",
    "Proposal to reduce platform fees by 25%",
    target_contract_address,
    calldata_for_fee_update,
    0
);

// Vote on the proposal
governance.cast_vote(proposal_id, 1, "I support this fee reduction");

// Delegate voting power
governance.delegate(trusted_delegate_address);

// Execute approved proposal
governance.execute_proposal(proposal_id);
```

## Testing

Comprehensive tests are provided in `tests/test_governance.cairo` covering:

- Governance initialization
- Proposal creation and management
- Voting mechanisms and edge cases
- Delegation functionality
- Authorization and security checks
- Proposal state transitions
- Quorum and threshold validation

Run tests with:

```bash
scarb test
```

## Best Practices

1. **Proposal Creation**: Ensure clear, detailed descriptions and proper
   target/calldata
2. **Voting**: Participate actively and consider delegation to knowledgeable
   delegates
3. **Delegation**: Choose delegates carefully and monitor their voting behavior
4. **Parameter Updates**: Use governance for critical parameter changes
5. **Security**: Verify proposal details before voting and execution

## Future Enhancements

- Signature verification for meta-transactions
- Historical balance tracking for precise voting power snapshots
- Multi-signature execution for critical proposals
- Proposal categorization and specialized voting rules
- Integration with other platform contracts

This governance system provides a robust foundation for decentralized
decision-making in the TeachLink platform while maintaining security and
transparency.
