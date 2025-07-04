mod StakingCalculation {
    pub fn calculate_reward_and_penalty(
        amount: u128, duration: u64, elapsed: u64
    ) -> (u128, u128) {
        if elapsed >= duration {
            let reward = amount / 10; // 10% reward
            return (reward, 0);
        } else {
            let penalty = ((duration - elapsed) * amount) / duration / 5;
            return (0, penalty);
        }
    }
}
