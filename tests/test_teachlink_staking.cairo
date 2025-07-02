#[test]
fn test_stake_and_withdraw() {
    // Simulate: stake 100 tokens, wait full duration, withdraw
    // Assert: stake is recorded, reward is given, withdraw updates state
}

#[test]
fn test_early_withdraw_penalty() {
    // Simulate: stake 100 tokens, withdraw early
    // Assert: penalty applied, no reward
}

#[test]
fn test_emergency_withdraw() {
    // Simulate: stake 100 tokens, emergency withdraw
    // Assert: only principal returned
}
