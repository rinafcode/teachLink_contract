# TeachLink Contract Development Tracking

This document tracks items that are planned for future development. These items were previously marked as `TODO` or `FIXME` in the codebase and have been moved here for proper tracking and prioritization.

## High Priority
- **Insurance Module (`escrow.rs`)**: Implement insurance capabilities for escrows to provide protection against default or failure scenarios.
- **Score Module (`lib.rs`, `test_score.rs`)**: Fully implement and integrate the credit scoring module for users based on their on-chain activities and contributions.
- **XDR Parsing (`horizon.service.ts`)**: Enhance the XDR parsing functionality in the indexer to support comprehensive event decoding for Soroban smart contracts.

## Medium Priority
- **Governance Module (`lib.rs`)**: Implement decentralized governance allowing token holders to vote on protocol upgrades, fee structures, and new chain support.
- **Provenance Module (`tokenization.rs`, `lib.rs`)**: Implement module to track content historical owners, full chain-of-custody, and origin validation.
- **Testutils Dependencies**: Re-enable `notification_tests` and ensure the `testutils` dependencies function appropriately without linking issues.
- **Reputation Module Tests (`test_reputation.rs`)**: Re-enable and expand automated tests for the reputation module once its interfaces are fully stabilized.
- **Escrow Tests (`test_escrow.rs.disabled`)**: Re-enable escrow testing when the insurance dependencies and time-lock simulations are robustly integrated.

## Low Priority
- **Automated Fuzz Testing Parsers (`test_generator.rs`)**: Finalize the parsing logic for inputs during fuzz testing to ensure appropriate types are passed to arbitrary functions.
- **Historical Owners Tracking (`tokenization.rs`)**: Integrate `provenance` module with tokenization to store and query historical ownership paths directly into token queries if needed.
