
# TeachLink Testing Sandbox

> Issue #381 — Comprehensive local testing environment

The sandbox gives every developer a fully isolated, reproducible
environment to write and run tests — no testnet, no real keys, no
waiting for network confirmations.

---

## Quick Start

```bash
# One command — starts everything, runs tests, cleans up
./scripts/sandbox.sh

# Run tests only (no Docker required)
./scripts/sandbox.sh --no-docker

# Keep the local node running after tests (for manual exploration)
./scripts/sandbox.sh --keep-up
```

---

## What the Sandbox Provides

| Feature | Description |
|---|---|
| Local Stellar node | Full Soroban-enabled node, no testnet needed |
| Mock token | SEP-41 compatible token — mint freely in tests |
| Named test accounts | `alice`, `bob`, `carol`, `dave` — readable tests |
| Time control | `advance_time()` and `advance_ledger()` helpers |
| Isolated environment | Every test gets a clean slate |
| Fast iteration | No network delay — tests run in milliseconds |

---

## Using the Sandbox in Your Tests

```rust
use crate::testing::sandbox::SandboxEnv;
use crate::testing::sandbox::fixtures::amounts;

#[test]
fn test_reward_distribution() {
    // 1. Create a fresh sandbox
    let sb = SandboxEnv::new();

    // 2. Use named accounts
    let educator = sb.accounts.bob();
    let learner  = sb.accounts.alice();

    // 3. Use amount constants
    let reward = amounts::TEN_XLM;

    // 4. Advance time if your contract needs it
    sb.advance_time(fixtures::time::ONE_DAY);

    // 5. Write assertions normally
    assert!(reward > 0);
}
```

---

## Using the Mock Token

```rust
use crate::testing::sandbox::{SandboxEnv, mock_token::MockToken};

#[test]
fn test_escrow_with_token() {
    let sb = SandboxEnv::new();

    // Deploy the mock token
    let token_id = sb.env.register_contract(None, MockToken);
    let token = MockTokenClient::new(&sb.env, &token_id);

    // Initialize it
    token.initialize(
        &sb.accounts.carol(),  // carol = admin
        &7u32,
        &String::from_str(&sb.env, "TeachLink Token"),
        &String::from_str(&sb.env, "TLT"),
    );

    // Mint tokens freely — no real XLM needed
    token.mint(&sb.accounts.carol(), &sb.accounts.alice(), &1_000_0000000i128);

    assert_eq!(token.balance(&sb.accounts.alice()), 1_000_0000000i128);
}
```

---

## File Structure