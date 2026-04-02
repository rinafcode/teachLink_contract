# TeachLink Smart Contract - Event Schema Documentation

## Overview

This document provides comprehensive documentation for all events emitted by the TeachLink smart contract. Events are emitted for all state changes to ensure full auditability and monitoring capabilities.

## Event Categories

### 1. Bridge Events

Events related to cross-chain bridge operations.

#### BridgeInitiatedEvent
Emitted when a bridge transaction is initiated.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Unique transaction identifier |
| `transaction` | `BridgeTransaction` | Transaction details |

#### BridgeCompletedEvent
Emitted when a bridge transaction is completed.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `message` | `CrossChainMessage` | Cross-chain message details |

#### BridgeCancelledEvent
Emitted when a bridge transaction is cancelled and refunded.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `refunded_to` | `Address` | Recipient of the refund |
| `amount` | `i128` | Refunded amount |
| `cancelled_at` | `u64` | Cancellation timestamp |

#### BridgeFailedEvent
Emitted when a bridge transaction fails.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `reason` | `Bytes` | Failure reason |
| `failed_at` | `u64` | Failure timestamp |

#### BridgeRetryEvent
Emitted when a bridge transaction is retried.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `retry_count` | `u32` | Current retry attempt number |
| `retried_at` | `u64` | Retry timestamp |

#### DepositEvent
Emitted when tokens are deposited for bridging.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `from` | `Address` | Depositor address |
| `amount` | `i128` | Deposited amount |
| `destination_chain` | `u32` | Destination chain ID |
| `destination_address` | `Bytes` | Destination address |

#### ReleaseEvent
Emitted when tokens are released from bridging.

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `u64` | Transaction identifier |
| `recipient` | `Address` | Recipient address |
| `amount` | `i128` | Released amount |
| `source_chain` | `u32` | Source chain ID |

#### ValidatorAddedEvent
Emitted when a validator is added.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `added_by` | `Address` | Admin who added |
| `added_at` | `u64` | Addition timestamp |

#### ValidatorRemovedEvent
Emitted when a validator is removed.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `removed_by` | `Address` | Admin who removed |
| `removed_at` | `u64` | Removal timestamp |

#### ChainSupportedEvent
Emitted when a chain is added to supported chains.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `added_by` | `Address` | Admin who added |
| `added_at` | `u64` | Addition timestamp |

#### ChainUnsupportedEvent
Emitted when a chain is removed from supported chains.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `removed_by` | `Address` | Admin who removed |
| `removed_at` | `u64` | Removal timestamp |

#### BridgeFeeUpdatedEvent
Emitted when the bridge fee is updated.

| Field | Type | Description |
|-------|------|-------------|
| `old_fee` | `i128` | Previous fee |
| `new_fee` | `i128` | New fee |
| `updated_by` | `Address` | Admin who updated |
| `updated_at` | `u64` | Update timestamp |

#### FeeRecipientUpdatedEvent
Emitted when the fee recipient is updated.

| Field | Type | Description |
|-------|------|-------------|
| `old_recipient` | `Address` | Previous recipient |
| `new_recipient` | `Address` | New recipient |
| `updated_by` | `Address` | Admin who updated |
| `updated_at` | `u64` | Update timestamp |

#### MinValidatorsUpdatedEvent
Emitted when minimum validator requirement is updated.

| Field | Type | Description |
|-------|------|-------------|
| `old_min` | `u32` | Previous minimum |
| `new_min` | `u32` | New minimum |
| `updated_by` | `Address` | Admin who updated |
| `updated_at` | `u64` | Update timestamp |

### 2. BFT Consensus Events

Events related to consensus operations.

#### ProposalCreatedEvent
Emitted when a new proposal is created.

| Field | Type | Description |
|-------|------|-------------|
| `proposal_id` | `u64` | Proposal identifier |
| `message` | `CrossChainMessage` | Proposal message |
| `required_votes` | `u32` | Votes needed for approval |

#### ProposalVotedEvent
Emitted when a validator votes on a proposal.

| Field | Type | Description |
|-------|------|-------------|
| `proposal_id` | `u64` | Proposal identifier |
| `validator` | `Address` | Voting validator |
| `vote` | `bool` | Vote value (true/false) |
| `vote_count` | `u32` | Current vote count |

#### ProposalExecutedEvent
Emitted when a proposal is executed.

| Field | Type | Description |
|-------|------|-------------|
| `proposal_id` | `u64` | Proposal identifier |
| `status` | `ProposalStatus` | Final status |
| `executed_at` | `u64` | Execution timestamp |

#### ValidatorRegisteredEvent
Emitted when a validator registers.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `stake` | `i128` | Initial stake |
| `joined_at` | `u64` | Registration timestamp |

#### ValidatorUnregisteredEvent
Emitted when a validator unregisters.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `unstaked_amount` | `i128` | Unstaked amount |
| `left_at` | `u64` | Unregistration timestamp |

### 3. Slashing and Rewards Events

#### ValidatorSlashedEvent
Emitted when a validator is slashed.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `amount` | `i128` | Slashed amount |
| `reason` | `SlashingReason` | Reason for slashing |
| `timestamp` | `u64` | Slashing timestamp |

#### ValidatorRewardedEvent
Emitted when a validator is rewarded.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `amount` | `i128` | Reward amount |
| `reward_type` | `RewardType` | Type of reward |
| `timestamp` | `u64` | Reward timestamp |

#### StakeDepositedEvent
Emitted when stake is deposited.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `amount` | `i128` | Deposited amount |
| `total_stake` | `i128` | Total stake after deposit |

#### StakeWithdrawnEvent
Emitted when stake is withdrawn.

| Field | Type | Description |
|-------|------|-------------|
| `validator` | `Address` | Validator address |
| `amount` | `i128` | Withdrawn amount |
| `remaining_stake` | `i128` | Remaining stake |

#### RewardPoolFundedExternalEvent
Emitted when the reward pool is funded externally.

| Field | Type | Description |
|-------|------|-------------|
| `funder` | `Address` | Funder address |
| `amount` | `i128` | Funded amount |
| `new_balance` | `i128` | New pool balance |
| `funded_at` | `u64` | Funding timestamp |

### 4. Emergency and Security Events

#### BridgePausedEvent
Emitted when the bridge is paused.

| Field | Type | Description |
|-------|------|-------------|
| `paused_by` | `Address` | Pauser address |
| `reason` | `Bytes` | Pause reason |
| `paused_at` | `u64` | Pause timestamp |
| `affected_chains` | `Vec<u32>` | Affected chain IDs |

#### BridgeResumedEvent
Emitted when the bridge is resumed.

| Field | Type | Description |
|-------|------|-------------|
| `resumed_by` | `Address` | Resumer address |
| `resumed_at` | `u64` | Resume timestamp |
| `affected_chains` | `Vec<u32>` | Affected chain IDs |

#### CircuitBreakerTriggeredEvent
Emitted when a circuit breaker is triggered.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `trigger_reason` | `Bytes` | Trigger reason |
| `triggered_at` | `u64` | Trigger timestamp |

#### CircuitBreakerInitializedEvent
Emitted when a circuit breaker is initialized.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `max_daily_volume` | `i128` | Maximum daily volume |
| `max_transaction_amount` | `i128` | Maximum transaction amount |
| `initialized_at` | `u64` | Initialization timestamp |

#### CircuitBreakerResetEvent
Emitted when a circuit breaker is reset.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `reset_by` | `Address` | Resetter address |
| `reset_at` | `u64` | Reset timestamp |

#### CircuitBreakerLimitsUpdatedEvent
Emitted when circuit breaker limits are updated.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `old_max_daily_volume` | `i128` | Previous daily volume limit |
| `new_max_daily_volume` | `i128` | New daily volume limit |
| `old_max_transaction_amount` | `i128` | Previous transaction limit |
| `new_max_transaction_amount` | `i128` | New transaction limit |
| `updated_by` | `Address` | Updater address |
| `updated_at` | `u64` | Update timestamp |

### 5. Escrow Events

#### EscrowCreatedEvent
Emitted when an escrow is created.

| Field | Type | Description |
|-------|------|-------------|
| `escrow` | `Escrow` | Escrow details |

#### EscrowApprovedEvent
Emitted when an escrow is approved.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `signer` | `Address` | Approving signer |
| `approval_count` | `u32` | Total approvals |

#### EscrowReleasedEvent
Emitted when an escrow is released.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `beneficiary` | `Address` | Beneficiary address |
| `amount` | `i128` | Released amount |

#### EscrowRefundedEvent
Emitted when an escrow is refunded.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `depositor` | `Address` | Depositor address |
| `amount` | `i128` | Refunded amount |

#### EscrowCancelledEvent
Emitted when an escrow is cancelled.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `depositor` | `Address` | Depositor address |
| `amount` | `i128` | Cancelled amount |
| `cancelled_at` | `u64` | Cancellation timestamp |

#### EscrowDisputedEvent
Emitted when an escrow is disputed.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `disputer` | `Address` | Disputer address |
| `reason` | `Bytes` | Dispute reason |

#### EscrowResolvedEvent
Emitted when an escrow dispute is resolved.

| Field | Type | Description |
|-------|------|-------------|
| `escrow_id` | `u64` | Escrow identifier |
| `outcome` | `DisputeOutcome` | Resolution outcome |
| `status` | `EscrowStatus` | Final status |

### 6. Insurance Events

#### InsurancePoolInitializedEvent
Emitted when the insurance pool is initialized.

| Field | Type | Description |
|-------|------|-------------|
| `token` | `Address` | Token address |
| `premium_rate` | `u32` | Premium rate (basis points) |
| `initialized_at` | `u64` | Initialization timestamp |

#### InsurancePoolFundedEvent
Emitted when the insurance pool is funded.

| Field | Type | Description |
|-------|------|-------------|
| `funder` | `Address` | Funder address |
| `amount` | `i128` | Funded amount |
| `new_balance` | `i128` | New pool balance |
| `funded_at` | `u64` | Funding timestamp |

#### InsurancePremiumPaidEvent
Emitted when an insurance premium is paid.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `amount` | `i128` | Premium amount |
| `paid_at` | `u64` | Payment timestamp |

#### InsuranceClaimProcessedEvent
Emitted when an insurance claim is processed.

| Field | Type | Description |
|-------|------|-------------|
| `recipient` | `Address` | Recipient address |
| `payout_amount` | `i128` | Payout amount |
| `new_balance` | `i128` | New pool balance |
| `processed_at` | `u64` | Processing timestamp |

### 7. Reputation Events

#### ParticipationUpdatedEvent
Emitted when user participation score is updated.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `points_added` | `u32` | Points added |
| `new_participation_score` | `u32` | New total score |
| `updated_at` | `u64` | Update timestamp |

#### CourseProgressUpdatedEvent
Emitted when user course progress is updated.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `total_courses_started` | `u32` | Total courses started |
| `total_courses_completed` | `u32` | Total courses completed |
| `completion_rate` | `u32` | Completion rate (basis points) |
| `updated_at` | `u64` | Update timestamp |

#### ContributionRatedEvent
Emitted when a user's contribution is rated.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `rating` | `u32` | Rating value |
| `new_contribution_quality` | `u32` | New average quality |
| `total_contributions` | `u32` | Total contributions |
| `rated_at` | `u64` | Rating timestamp |

### 8. Content Tokenization Events

#### ContentMintedEvent
Emitted when content is minted as NFT.

| Field | Type | Description |
|-------|------|-------------|
| `token_id` | `u64` | Token identifier |
| `creator` | `Address` | Creator address |
| `metadata` | `ContentMetadata` | Content metadata |

#### OwnershipTransferredEvent
Emitted when token ownership is transferred.

| Field | Type | Description |
|-------|------|-------------|
| `token_id` | `u64` | Token identifier |
| `from` | `Address` | Previous owner |
| `to` | `Address` | New owner |
| `timestamp` | `u64` | Transfer timestamp |

#### ProvenanceRecordedEvent
Emitted when provenance is recorded.

| Field | Type | Description |
|-------|------|-------------|
| `token_id` | `u64` | Token identifier |
| `record` | `ProvenanceRecord` | Provenance details |

#### MetadataUpdatedEvent
Emitted when token metadata is updated.

| Field | Type | Description |
|-------|------|-------------|
| `token_id` | `u64` | Token identifier |
| `owner` | `Address` | Owner address |
| `timestamp` | `u64` | Update timestamp |

#### TransferabilityUpdatedEvent
Emitted when token transferability is updated.

| Field | Type | Description |
|-------|------|-------------|
| `token_id` | `u64` | Token identifier |
| `owner` | `Address` | Owner address |
| `transferable` | `bool` | Transferability status |
| `updated_at` | `u64` | Update timestamp |

### 9. Rewards Events

#### RewardIssuedEvent
Emitted when a reward is issued.

| Field | Type | Description |
|-------|------|-------------|
| `recipient` | `Address` | Recipient address |
| `amount` | `i128` | Reward amount |
| `reward_type` | `String` | Reward type |
| `timestamp` | `u64` | Issuance timestamp |

#### RewardClaimedEvent
Emitted when a reward is claimed.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `amount` | `i128` | Claimed amount |
| `timestamp` | `u64` | Claim timestamp |

#### RewardPoolFundedEvent
Emitted when the reward pool is funded.

| Field | Type | Description |
|-------|------|-------------|
| `funder` | `Address` | Funder address |
| `amount` | `i128` | Funded amount |
| `timestamp` | `u64` | Funding timestamp |

### 10. Multi-Chain and Liquidity Events

#### ChainAddedEvent
Emitted when a chain is added.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `chain_name` | `Bytes` | Chain name |
| `added_at` | `u64` | Addition timestamp |

#### ChainUpdatedEvent
Emitted when a chain configuration is updated.

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `u32` | Chain identifier |
| `is_active` | `bool` | Active status |
| `updated_at` | `u64` | Update timestamp |

#### AssetRegisteredEvent
Emitted when an asset is registered.

| Field | Type | Description |
|-------|------|-------------|
| `asset_id` | `Bytes` | Asset identifier |
| `stellar_token` | `Address` | Stellar token address |
| `supported_chains` | `u32` | Number of supported chains |

#### LiquidityAddedEvent
Emitted when liquidity is added.

| Field | Type | Description |
|-------|------|-------------|
| `provider` | `Address` | Liquidity provider |
| `chain_id` | `u32` | Chain identifier |
| `amount` | `i128` | Added amount |
| `share_percentage` | `u32` | Share percentage |

#### LiquidityRemovedEvent
Emitted when liquidity is removed.

| Field | Type | Description |
|-------|------|-------------|
| `provider` | `Address` | Liquidity provider |
| `chain_id` | `u32` | Chain identifier |
| `amount` | `i128` | Removed amount |
| `rewards` | `i128` | Rewards earned |

#### FeeUpdatedEvent
Emitted when fees are updated.

| Field | Type | Description |
|-------|------|-------------|
| `old_fee` | `i128` | Previous fee |
| `new_fee` | `i128` | New fee |
| `multiplier` | `u32` | Fee multiplier |

### 11. Message Passing Events

#### PacketSentEvent
Emitted when a packet is sent.

| Field | Type | Description |
|-------|------|-------------|
| `packet_id` | `u64` | Packet identifier |
| `source_chain` | `u32` | Source chain ID |
| `destination_chain` | `u32` | Destination chain ID |
| `sender` | `Bytes` | Sender address |
| `nonce` | `u64` | Nonce |

#### PacketDeliveredEvent
Emitted when a packet is delivered.

| Field | Type | Description |
|-------|------|-------------|
| `packet_id` | `u64` | Packet identifier |
| `delivered_at` | `u64` | Delivery timestamp |
| `gas_used` | `u64` | Gas consumed |

#### PacketFailedEvent
Emitted when a packet delivery fails.

| Field | Type | Description |
|-------|------|-------------|
| `packet_id` | `u64` | Packet identifier |
| `reason` | `Bytes` | Failure reason |
| `failed_at` | `u64` | Failure timestamp |

### 12. Credit Score Events

#### CreditScoreUpdatedEvent
Emitted when a credit score is updated.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `new_score` | `u64` | New credit score |

#### CourseCompletedEvent
Emitted when a course is completed.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `course_id` | `u64` | Course identifier |
| `points` | `u64` | Points earned |

#### ContributionRecordedEvent
Emitted when a contribution is recorded.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `Address` | User address |
| `c_type` | `ContributionType` | Contribution type |
| `points` | `u64` | Points earned |

### 13. Analytics and Reporting Events

#### ReportGeneratedEvent
Emitted when a report is generated.

| Field | Type | Description |
|-------|------|-------------|
| `report_id` | `u64` | Report identifier |
| `report_type` | `ReportType` | Report type |
| `generated_by` | `Address` | Generator address |
| `period_start` | `u64` | Period start |
| `period_end` | `u64` | Period end |

#### ReportScheduledEvent
Emitted when a report is scheduled.

| Field | Type | Description |
|-------|------|-------------|
| `schedule_id` | `u64` | Schedule identifier |
| `template_id` | `u64` | Template identifier |
| `owner` | `Address` | Owner address |
| `next_run_at` | `u64` | Next run timestamp |

#### ReportCommentAddedEvent
Emitted when a comment is added to a report.

| Field | Type | Description |
|-------|------|-------------|
| `report_id` | `u64` | Report identifier |
| `comment_id` | `u64` | Comment identifier |
| `author` | `Address` | Author address |

#### AlertTriggeredEvent
Emitted when an alert is triggered.

| Field | Type | Description |
|-------|------|-------------|
| `rule_id` | `u64` | Rule identifier |
| `condition_type` | `AlertConditionType` | Condition type |
| `current_value` | `i128` | Current value |
| `threshold` | `i128` | Threshold value |
| `triggered_at` | `u64` | Trigger timestamp |

### 14. Backup and Recovery Events

#### BackupCreatedEvent
Emitted when a backup is created.

| Field | Type | Description |
|-------|------|-------------|
| `backup_id` | `u64` | Backup identifier |
| `created_by` | `Address` | Creator address |
| `integrity_hash` | `Bytes` | Integrity hash |
| `rto_tier` | `RtoTier` | RTO tier |
| `created_at` | `u64` | Creation timestamp |

#### BackupVerifiedEvent
Emitted when a backup is verified.

| Field | Type | Description |
|-------|------|-------------|
| `backup_id` | `u64` | Backup identifier |
| `verified_by` | `Address` | Verifier address |
| `verified_at` | `u64` | Verification timestamp |
| `valid` | `bool` | Verification result |

#### RecoveryExecutedEvent
Emitted when a recovery is executed.

| Field | Type | Description |
|-------|------|-------------|
| `recovery_id` | `u64` | Recovery identifier |
| `backup_id` | `u64` | Backup identifier |
| `executed_by` | `Address` | Executor address |
| `recovery_duration_secs` | `u64` | Recovery duration |
| `success` | `bool` | Success status |

### 15. Atomic Swap Events

#### SwapInitiatedEvent
Emitted when a swap is initiated.

| Field | Type | Description |
|-------|------|-------------|
| `swap_id` | `u64` | Swap identifier |
| `initiator` | `Address` | Initiator address |
| `initiator_amount` | `i128` | Initiator amount |
| `counterparty` | `Address` | Counterparty address |
| `counterparty_amount` | `i128` | Counterparty amount |
| `timelock` | `u64` | Timelock timestamp |

#### SwapCompletedEvent
Emitted when a swap is completed.

| Field | Type | Description |
|-------|------|-------------|
| `swap_id` | `u64` | Swap identifier |
| `completed_at` | `u64` | Completion timestamp |

#### SwapRefundedEvent
Emitted when a swap is refunded.

| Field | Type | Description |
|-------|------|-------------|
| `swap_id` | `u64` | Swap identifier |
| `refunded_to` | `Address` | Refund recipient |
| `amount` | `i128` | Refunded amount |

### 16. Audit Events

#### AuditRecordCreatedEvent
Emitted when an audit record is created.

| Field | Type | Description |
|-------|------|-------------|
| `record_id` | `u64` | Record identifier |
| `operation_type` | `OperationType` | Operation type |
| `operator` | `Address` | Operator address |
| `timestamp` | `u64` | Operation timestamp |

### 17. Performance Events

#### PerfMetricsComputedEvent
Emitted when performance metrics are computed.

| Field | Type | Description |
|-------|------|-------------|
| `health_score` | `u32` | Health score |
| `computed_at` | `u64` | Computation timestamp |

#### PerfCacheInvalidatedEvent
Emitted when performance cache is invalidated.

| Field | Type | Description |
|-------|------|-------------|
| `invalidated_at` | `u64` | Invalidation timestamp |

## Event Naming Conventions

All events follow these naming conventions:

1. **PascalCase**: Event names use PascalCase (e.g., `BridgeInitiatedEvent`)
2. **Past Tense**: Event names use past tense verbs (e.g., `Created`, `Updated`, `Deleted`)
3. **Suffix**: All event names end with `Event`
4. **Descriptive**: Names clearly indicate the action and entity

## Event Filtering and Querying

Events can be filtered and queried using the following approaches:

### By Event Type
Filter events by their struct name (e.g., `BridgeInitiatedEvent`, `EscrowCreatedEvent`).

### By Field Value
Filter events by specific field values:
- By address: Filter events involving a specific `Address`
- By ID: Filter events with specific `nonce`, `escrow_id`, `token_id`, etc.
- By timestamp: Filter events within a time range using `*_at` fields

### By Category
Events are organized by functional category for easier querying:
- Bridge operations
- Consensus operations
- Slashing and rewards
- Emergency controls
- Escrow management
- Insurance operations
- Reputation tracking
- Content tokenization
- And more...

## Usage Examples

### Querying Bridge Events
```rust
// Get all bridge initiated events for a specific user
let events = env.events()
    .filter(|e| e.type_name == "BridgeInitiatedEvent")
    .filter(|e| e.transaction.recipient == user_address);
```

### Auditing Validator Activity
```rust
// Get all validator-related events
let validator_events = [
    "ValidatorRegisteredEvent",
    "ValidatorUnregisteredEvent",
    "ValidatorSlashedEvent",
    "ValidatorRewardedEvent"
];
```

### Tracking Escrow Lifecycle
```rust
// Get complete escrow history
let escrow_events = [
    "EscrowCreatedEvent",
    "EscrowApprovedEvent",
    "EscrowReleasedEvent",
    "EscrowRefundedEvent",
    "EscrowCancelledEvent",
    "EscrowDisputedEvent",
    "EscrowResolvedEvent"
];
```

## Best Practices

1. **Always Check Events**: Applications should monitor relevant events for state changes
2. **Handle All Event Types**: Implement handlers for all event types in your category
3. **Use Timestamps**: Use event timestamps for ordering and time-based queries
4. **Index by ID**: Index events by their primary IDs for efficient lookups
5. **Monitor Admin Actions**: Pay special attention to admin/configuration change events

## Version History

- **v1.0**: Initial event schema with comprehensive coverage of all state changes
- Events are added as new functionality is implemented

## Support

For questions or issues related to events, please refer to the main project documentation or open an issue on the project repository.
