#[contract]
mod TeachLinkStaking {
    use starknet::ContractAddress;
    use starknet::get_block_timestamp;
    use starknet::get_caller_address;
    use src::contracts::staking::libraries::StakingCalculation;

    struct Stake {
        amount: u128,
        start_time: u64,
        duration: u64,
        has_withdrawn: bool,
    }

    #[storage]
    struct Storage {
        stakes: LegacyMap<ContractAddress, Stake>,
        total_staked: u128,
        reward_pool: u128,
    }

    #[external]
    fn stake(amount: u128, duration: u64) {
        let caller = get_caller_address();
        let now = get_block_timestamp();

        let stake = Stake {
            amount,
            start_time: now,
            duration,
            has_withdrawn: false,
        };

        stakes::write(caller, stake);
        total_staked::write(total_staked::read() + amount);
    }

    #[external]
    fn withdraw() {
        let caller = get_caller_address();
        let stake = stakes::read(caller);
        assert(!stake.has_withdrawn, 'Already withdrawn');

        let now = get_block_timestamp();
        let elapsed = now - stake.start_time;

        let (reward, penalty) = StakingCalculation::calculate_reward_and_penalty(
            stake.amount, stake.duration, elapsed
        );

        let final_amount = stake.amount - penalty + reward;

        // Simulate payout logic
        // transfer(caller, final_amount);

        stakes::write(caller, Stake { has_withdrawn: true, ..stake });
        total_staked::write(total_staked::read() - stake.amount);
    }

    #[external]
    fn emergency_withdraw() {
        let caller = get_caller_address();
        let stake = stakes::read(caller);
        assert(!stake.has_withdrawn, 'Already withdrawn');

        // Only return staked amount (no rewards, no penalties)
        // transfer(caller, stake.amount);

        stakes::write(caller, Stake { has_withdrawn: true, ..stake });
        total_staked::write(total_staked::read() - stake.amount);
    }
}
