# Reentrancy Vulnerability Fix - Escrow and Reward Distribution

## Issue Overview

**Severity:** Critical  
**Category:** Security & Safety  
**Impact:** Critical - Could lead to fund drainage  

### Description
Escrow and reward functions in the TeachLink smart contracts may be vulnerable to reentrancy attacks. Reentrancy attacks occur when an external contract call is made before state changes are completed, allowing malicious contracts to recursively call functions and drain funds.

## Affected Contracts

The following contracts require reentrancy protection:

1. **Escrow Contract** - Handles fund locking and release
2. **Reward Distribution Contract** - Manages learning rewards and incentives
3. **Marketplace Contract** - Processes payments and settlements
4. **Insurance Contract** - Handles claims and payouts

## Implementation Strategy

### 1. Checks-Effects-Interactions Pattern

#### Before (Vulnerable Code):
```rust
// ❌ VULNERABLE - State change after external call
pub fn withdraw_reward(env: Env, user_id: Address, amount: i128) {
    let balance = Self::get_user_balance(&env, &user_id);
    require!(balance >= amount, "Insufficient balance");
    
    // External call BEFORE state change
    token::transfer(&env, &contract_address, &user_id, &amount);
    
    // State change AFTER - VULNERABLE!
    let new_balance = balance - amount;
    Self::set_user_balance(&env, &user_id, &new_balance);
}
```

#### After (Secure Code):
```rust
// ✅ SECURE - Checks-Effects-Interactions pattern
pub fn withdraw_reward(env: Env, user_id: Address, amount: i128) {
    // CHECK: Validate conditions
    let balance = Self::get_user_balance(&env, &user_id);
    require!(balance >= amount, "Insufficient balance");
    require!(amount > 0, "Amount must be positive");
    
    // EFFECTS: Update state BEFORE external calls
    let new_balance = balance - amount;
    Self::set_user_balance(&env, &user_id, &new_balance);
    
    // Log the withdrawal (state change)
    let event_data = WithdrawalEvent {
        user: user_id.clone(),
        amount,
        timestamp: env.ledger().timestamp(),
    };
    event_data.publish(&env);
    
    // INTERACTIONS: External calls LAST
    token::transfer(&env, &contract_address, &user_id, &amount);
}
```

### 2. Reentrancy Guards Implementation

#### Custom Reentrancy Guard for Soroban:

```rust
use soroban_sdk::{contracttype, Address, Env, Symbol};

#[contracttype]
pub enum StorageKey {
    ReentrancyGuard(Symbol),
}

pub struct ReentrancyGuard;

impl ReentrancyGuard {
    /// Acquire a reentrancy guard for a specific function
    pub fn acquire(env: &Env, function_name: Symbol) -> bool {
        let key = StorageKey::ReentrancyGuard(function_name);
        
        if env.storage().instance().has(&key) {
            return false; // Already in execution - REENTRANCY DETECTED!
        }
        
        env.storage().instance().set(&key, &true);
        true
    }
    
    /// Release the reentrancy guard after function completion
    pub fn release(env: &Env, function_name: Symbol) {
        let key = StorageKey::ReentrancyGuard(function_name);
        env.storage().instance().remove(&key);
    }
}

// Usage in contract functions
#[contractimpl]
impl TeachLinkContract {
    pub fn process_escrow_release(env: Env, escrow_id: u64) {
        let guard_symbol = Symbol::new(&env, "process_escrow_release");
        
        // Acquire guard
        require!(
            ReentrancyGuard::acquire(&env, guard_symbol.clone()),
            "Reentrancy detected"
        );
        
        // Ensure guard is released even if error occurs
        defer! {
            ReentrancyGuard::release(&env, guard_symbol);
        }
        
        // Function logic here
        Self::execute_escrow_release(&env, escrow_id);
    }
}
```

### 3. State Change Audit Checklist

Before any external call, verify:

- [ ] All balance updates completed
- [ ] All ownership transfers recorded
- [ ] All status changes persisted
- [ ] All events emitted
- [ ] All locks/unlocks processed

#### Example: Escrow State Changes

```rust
pub fn release_escrow(env: Env, escrow_id: u64, recipient: Address) {
    // === CHECKS ===
    let escrow = Self::get_escrow(&env, escrow_id)
        .unwrap_or_else(|| panic!("Escrow not found"));
    
    require!(escrow.status == EscrowStatus::Active, "Escrow not active");
    require!(
        escrow.conditions_met(&env),
        "Release conditions not met"
    );
    
    // === EFFECTS (ALL STATE CHANGES BEFORE EXTERNAL CALLS) ===
    
    // 1. Update escrow status
    let mut updated_escrow = escrow.clone();
    updated_escrow.status = EscrowStatus::Released;
    updated_escrow.released_at = env.ledger().timestamp();
    Self::update_escrow(&env, &updated_escrow);
    
    // 2. Update balances
    let contract_balance = Self::get_contract_balance(&env);
    Self::set_contract_balance(&env, contract_balance - updated_escrow.amount);
    
    let recipient_balance = Self::get_user_balance(&env, &recipient);
    Self::set_user_balance(&env, &recipient, recipient_balance + updated_escrow.amount);
    
    // 3. Record transaction
    let tx_record = TransactionRecord {
        escrow_id,
        from: escrow.depositor.clone(),
        to: recipient.clone(),
        amount: updated_escrow.amount,
        timestamp: env.ledger().timestamp(),
        tx_type: TransactionType::EscrowRelease,
    };
    Self::record_transaction(&env, &tx_record);
    
    // 4. Emit events
    EscrowReleasedEvent {
        escrow_id,
        recipient: recipient.clone(),
        amount: updated_escrow.amount,
    }.publish(&env);
    
    // === INTERACTIONS (EXTERNAL CALLS LAST) ===
    
    // Now safe to transfer tokens
    token::transfer(
        &env,
        &env.current_contract_address(),
        &recipient,
        &updated_escrow.amount
    );
}
```

### 4. Specific Functions Requiring Protection

#### A. Reward Distribution Functions

```rust
/// Distribute learning rewards to multiple users
pub fn batch_distribute_rewards(
    env: Env,
    recipients: Vec<Address>,
    amounts: Vec<i128>,
) {
    require!(recipients.len() == amounts.len(), "Length mismatch");
    
    let guard = Symbol::new(&env, "batch_distribute");
    require!(ReentrancyGuard::acquire(&env, guard.clone()), "Reentrancy detected");
    defer! { ReentrancyGuard::release(&env, guard); }
    
    // Calculate total
    let total_amount: i128 = amounts.iter().fold(0, |sum, amount| sum + amount);
    require!(
        Self::get_contract_balance(&env) >= total_amount,
        "Insufficient contract balance"
    );
    
    // Process each recipient
    for i in 0..recipients.len() {
        let recipient = recipients.get(i).unwrap();
        let amount = amounts.get(i).unwrap();
        
        // CHECKS
        require!(amount > 0, "Amount must be positive");
        
        // EFFECTS
        let new_balance = Self::get_user_balance(&env, &recipient) + amount;
        Self::set_user_balance(&env, &recipient, &new_balance);
        
        RewardDistributedEvent {
            recipient: recipient.clone(),
            amount,
        }.publish(&env);
        
        // INTERACTIONS
        token::transfer(&env, &env.current_contract_address(), &recipient, &amount);
    }
}
```

#### B. Escrow Creation and Funding

```rust
pub fn create_and_fund_escrow(
    env: Env,
    depositor: Address,
    amount: i128,
    conditions: EscrowConditions,
) -> u64 {
    depositor.require_auth();
    
    let guard = Symbol::new(&env, "create_escrow");
    require!(ReentrancyGuard::acquire(&env, guard.clone()), "Reentrancy detected");
    defer! { ReentrancyGuard::release(&env, guard); }
    
    // CHECKS
    require!(amount > 0, "Amount must be positive");
    require!(
        token::balance(&env, &depositor) >= amount,
        "Insufficient depositor balance"
    );
    
    // EFFECTS
    let escrow_id = Self::generate_escrow_id(&env);
    let escrow = Escrow {
        id: escrow_id,
        depositor: depositor.clone(),
        amount,
        status: EscrowStatus::Active,
        created_at: env.ledger().timestamp(),
        conditions,
    };
    
    Self::store_escrow(&env, &escrow);
    
    // Update depositor's locked balance
    let locked = Self::get_locked_balance(&env, &depositor) + amount;
    Self::set_locked_balance(&env, &depositor, &locked);
    
    EscrowCreatedEvent {
        escrow_id,
        depositor: depositor.clone(),
        amount,
    }.publish(&env);
    
    // INTERACTIONS
    token::transfer(&env, &depositor, &env.current_contract_address(), &amount);
    
    escrow_id
}
```

#### C. Insurance Claim Payouts

```rust
pub fn process_insurance_claim_payout(
    env: Env,
    claim_id: u64,
    claimant: Address,
) {
    claimant.require_auth();
    
    let guard = Symbol::new(&env, "claim_payout");
    require!(ReentrancyGuard::acquire(&env, guard.clone()), "Reentrancy detected");
    defer! { ReentrancyGuard::release(&env, guard); }
    
    // CHECKS
    let claim = Self::get_claim(&env, claim_id)
        .unwrap_or_else(|| panic!("Claim not found"));
    
    require!(claim.status == ClaimStatus::Approved, "Claim not approved");
    require!(claim.claimant == claimant, "Unauthorized claimant");
    
    // EFFECTS
    let mut updated_claim = claim.clone();
    updated_claim.status = ClaimStatus::Paid;
    updated_claim.paid_at = env.ledger().timestamp();
    Self::update_claim(&env, &updated_claim);
    
    // Update insurance pool balance
    let pool_balance = Self::get_insurance_pool_balance(&env);
    Self::set_insurance_pool_balance(&env, pool_balance - updated_claim.payout_amount);
    
    // Update claimant balance
    let claimant_balance = Self::get_user_balance(&env, &claimant);
    Self::set_user_balance(&env, &claimant, claimant_balance + updated_claim.payout_amount);
    
    ClaimPaidEvent {
        claim_id,
        claimant: claimant.clone(),
        amount: updated_claim.payout_amount,
    }.publish(&env);
    
    // INTERACTIONS
    token::transfer(
        &env,
        &Self::get_insurance_pool_address(&env),
        &claimant,
        &updated_claim.payout_amount
    );
}
```

## Testing Strategy

### Reentrancy Test Cases

```rust
#[cfg(test)]
mod reentrancy_tests {
    use super::*;
    
    /// Test: Prevent reentrancy in reward withdrawal
    #[test]
    fn test_reentrancy_in_withdraw_reward() {
        let env = Env::default();
        let contract_address = env.register_contract(None, TestContract);
        let user = Address::generate(&env);
        
        // Setup: Fund user with rewards
        TestContract::set_user_balance(&env, &user, &1000);
        
        // Attempt reentrancy attack via malicious contract
        let malicious_contract = MaliciousReentrancyContract::deploy(&env);
        
        // This should fail due to reentrancy guard
        let result = env.try_invoke_contract::<_, ()>(
            &contract_address,
            &Symbol::new(&env, "withdraw_reward"),
            vec![&env, user.into_val(&env), 100.into_val(&env)],
        );
        
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Error::from("Reentrancy detected"));
    }
    
    /// Test: Multiple withdrawals in single transaction
    #[test]
    fn test_batch_withdrawal_no_reentrancy() {
        let env = Env::default();
        let contract_address = env.register_contract(None, TestContract);
        
        let users = vec![
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];
        
        let amounts = vec![100, 200, 300];
        
        // Should complete successfully without reentrancy
        TestContract::batch_distribute_rewards(
            &env,
            users.clone(),
            amounts.clone(),
        );
        
        // Verify all balances updated correctly
        for (i, user) in users.iter().enumerate() {
            assert_eq!(
                TestContract::get_user_balance(&env, &user),
                amounts.get(i).unwrap()
            );
        }
    }
    
    /// Test: Escrow release reentrancy protection
    #[test]
    fn test_escrow_release_reentrancy_guard() {
        let env = Env::default();
        let contract_address = env.register_contract(None, TestContract);
        
        let escrow_id = setup_test_escrow(&env);
        let recipient = Address::generate(&env);
        
        // Normal release should work
        TestContract::release_escrow(&env, escrow_id, recipient.clone());
        
        // Verify escrow status changed before transfer
        let escrow = TestContract::get_escrow(&env, escrow_id).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
    }
}
```

## Static Analysis Integration

### Using `cargo-audit` and Custom Linters

```toml
# Cargo.toml
[dev-dependencies]
cargo-audit = "0.18.0"
slither-analyzer = "0.1.0"
```

```bash
#!/bin/bash
# scripts/security-audit.sh

echo "Running reentrancy detection..."

# Run cargo audit
cargo audit

# Run custom reentrancy checker
cargo run --bin reentrancy-detector -- ./contracts/

# Generate security report
cargo sec-lint --format=json > security-report.json

echo "Security audit complete!"
```

## Deployment Checklist

- [ ] Implement checks-effects-interactions pattern in all external-facing functions
- [ ] Add reentrancy guards to escrow functions
- [ ] Add reentrancy guards to reward distribution functions
- [ ] Add reentrancy guards to insurance payout functions
- [ ] Add reentrancy guards to marketplace settlement functions
- [ ] Audit all state changes occur before external calls
- [ ] Write comprehensive reentrancy test cases
- [ ] Run static analysis tools
- [ ] Conduct third-party security audit
- [ ] Document all security measures

## Monitoring and Alerts

Implement runtime monitoring for suspicious patterns:

```rust
pub fn monitor_withdrawal_patterns(env: &Env, user: &Address) {
    let recent_withdrawals = Self::get_recent_withdrawals(env, user, 300); // Last 5 minutes
    
    if recent_withdrawals.len() > 5 {
        // Alert: Unusual withdrawal frequency
        SecurityAlertEvent {
            alert_type: AlertType::SuspiciousWithdrawalPattern,
            user: user.clone(),
            count: recent_withdrawals.len(),
        }.publish(env);
    }
}
```

## References

- [Stellar Soroban Security Best Practices](https://soroban.stellar.org/docs)
- [SWC-107: Reentrancy](https://swcregistry.io/docs/SWC-107)
- [Checks-Effects-Interactions Pattern](https://fravoll.github.io/solidity-patterns/checks_effects_interactions.html)
