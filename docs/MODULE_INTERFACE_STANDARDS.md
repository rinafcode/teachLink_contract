# Module Interface Standards

Every Rust module in `contracts/teachlink/src/` must follow this standard.

---

## 1. Module-level doc comment

Every file must open with a `//!` doc comment that states:
- What the module does (one sentence)
- Key responsibilities (bullet list)

```rust
//! Reward pool management and distribution.
//!
//! Responsibilities:
//! - Initialize and fund the reward pool
//! - Issue rewards to users
//! - Allow users to claim pending rewards
//! - Expose read-only views for pool state
```

---

## 2. Manager-struct pattern

All public logic must live on a zero-size manager struct, not as free
functions. This makes call sites unambiguous and enables future trait
extraction.

```rust
// ✅ correct
pub struct RewardsManager;
impl RewardsManager {
    pub fn initialize(env: &Env, ...) -> Result<(), RewardsError> { ... }
}

// ❌ incorrect — free functions
pub fn initialize_rewards(env: &Env, ...) -> Result<(), RewardsError> { ... }
```

---

## 3. Section comments

Group methods with `// ===== Section Name =====` comments. Required
sections (use only those that apply):

```
// ===== Initialization =====
// ===== Mutations =====
// ===== Admin =====
// ===== Queries =====
```

---

## 4. `#[must_use]` on pure getters

Any method that returns a value and has no side effects must carry
`#[must_use]`.

```rust
#[must_use]
pub fn get_score(env: &Env, user: Address) -> u64 { ... }
```

---

## 5. Error handling

- State-changing functions return `Result<T, E>` where `E` is a typed
  error from `errors.rs`.
- Pure getters may return `T` or `Option<T>` directly.

---

## 6. Authorization pattern

Require auth at the top of every state-changing function that acts on
behalf of a user or admin. Use `#[cfg(not(test))]` guards only when
the test harness cannot provide auth.

```rust
pub fn update_participation(env: &Env, user: Address, points: u32) {
    user.require_auth();
    // ...
}
```

---

## 7. Compliant modules (reference implementations)

| Module | Manager struct | Module doc | Section comments | `#[must_use]` getters |
|---|---|---|---|---|
| `audit.rs` | `AuditManager` | ✅ | ✅ | — |
| `performance.rs` | `PerformanceManager` | ✅ | ✅ | — |
| `sustainability.rs` | `SustainabilityManager` | ✅ | ✅ | — |
| `reputation.rs` | `ReputationManager` | ✅ | ✅ | ✅ |
| `score.rs` | `ScoreManager` | ✅ | ✅ | ✅ |
| `rewards.rs` | `Rewards` | ✅ | ✅ | ✅ |

---

## 8. Minimal example

```rust
//! Example module following the interface standard.
//!
//! Responsibilities:
//! - Track a counter per user
//! - Expose a read-only view

use soroban_sdk::{Address, Env, Symbol, symbol_short};

const COUNTER: Symbol = symbol_short!("counter");

pub struct ExampleManager;

impl ExampleManager {
    // ===== Mutations =====

    pub fn increment(env: &Env, user: Address) {
        user.require_auth();
        let key = (COUNTER, user.clone());
        let n: u32 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(n + 1));
    }

    // ===== Queries =====

    #[must_use]
    pub fn get(env: &Env, user: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&(COUNTER, user))
            .unwrap_or(0)
    }
}
```
