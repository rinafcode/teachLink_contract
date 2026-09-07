# TeachLink Contract Development Tracking

This document tracks items that are planned for future development. These items were previously marked as `TODO` or `FIXME` in the codebase and have been moved here for proper tracking and prioritization.

## High Priority
- **XDR Parsing (`horizon.service.ts`)**: Enhance the XDR parsing functionality in the indexer to support comprehensive event decoding for Soroban smart contracts.
- **Governance Module (`lib.rs`)**: Implement decentralized governance allowing token holders to vote on protocol upgrades, fee structures, and new chain support.
- **Governance Module (`lib.rs`)**: Implement decentralized governance allowing token holders to vote on protocol upgrades, fee structures, and new chain support.
- **Governance Module (`lib.rs`)**: Implement decentralized governance allowing token holders to vote on protocol upgrades, fee structures, and new chain support.

## Medium Priority
- **Testutils Dependencies**: Re-enable `notification_tests` and ensure the `testutils` dependencies function appropriately without linking issues.
- **Event Accumulation Across Calls (`test_cross_contract_interactions.rs`)**: `test_event_multiple_modules_emit_events` is `#[ignore]`d - `env.events().all()` doesn't appear to accumulate events across three separate client calls the way the test assumes (only 1 event observed, expected >= 4). Needs investigation into soroban-sdk 25.x event-scoping semantics before re-enabling.

## Low Priority
- **Automated Fuzz Testing Parsers (`test_generator.rs`)**: Finalize the parsing logic for inputs during fuzz testing to ensure appropriate types are passed to arbitrary functions.
