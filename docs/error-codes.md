# Error Codes Documentation

> Closes #349

## Overview

This document describes all error codes used in the TeachLink smart contract, their meanings, common causes, and resolution steps.

---

## Error Reference

### `AlreadyInitialized`
- **Meaning**: The contract has already been initialized and cannot be initialized again.
- **Common Cause**: Calling `initialize` more than once on the same contract instance.
- **Resolution**: Check if the contract is already deployed and initialized before calling `initialize`.

### `Unauthorized`
- **Meaning**: The caller does not have permission to perform the requested action.
- **Common Cause**: Invoking an admin-only function with a non-admin address.
- **Resolution**: Ensure the transaction is signed by the correct authorized account.

### `InvalidAmount`
- **Meaning**: The provided token amount is zero or negative.
- **Common Cause**: Passing `0` or a negative value where a positive amount is required.
- **Resolution**: Validate that the amount is greater than zero before calling the function.
- **Example**:
  ```rust
  // Bad
  contract.reward(env, learner, 0);

  // Good
  contract.reward(env, learner, 100);
  ```

### `InsufficientBalance`
- **Meaning**: The account does not hold enough tokens to complete the operation.
- **Common Cause**: Attempting to transfer or spend more tokens than the account balance.
- **Resolution**: Query the account balance first and ensure it covers the requested amount.

### `LearnerNotFound`
- **Meaning**: The specified learner address is not registered in the contract.
- **Common Cause**: Calling learner-specific functions before the learner has been registered.
- **Resolution**: Register the learner with the appropriate onboarding function before interacting.

### `CourseNotFound`
- **Meaning**: The referenced course ID does not exist in contract storage.
- **Common Cause**: Using a stale or incorrect course ID.
- **Resolution**: Retrieve the list of active courses and verify the ID before use.

### `EscrowNotFound`
- **Meaning**: No escrow record exists for the given identifier.
- **Common Cause**: Querying or releasing an escrow that was never created or has already been settled.
- **Resolution**: Confirm the escrow was created and is still active before attempting to release it.

### `OverflowError`
- **Meaning**: An arithmetic operation would exceed the maximum value for the type.
- **Common Cause**: Accumulating very large reward totals or token supplies.
- **Resolution**: Use checked arithmetic and validate inputs to stay within safe bounds.

---

## Error Handling Pattern

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized  = 1,
    Unauthorized        = 2,
    InvalidAmount       = 3,
    InsufficientBalance = 4,
    LearnerNotFound     = 5,
    CourseNotFound      = 6,
    EscrowNotFound      = 7,
    OverflowError       = 8,
}
```

All contract functions return `Result<T, Error>`. Callers should match on the error variant to provide meaningful feedback to end users.
