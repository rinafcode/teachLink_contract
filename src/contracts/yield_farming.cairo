#[starknet::contract]
pub mod YieldFarming {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        contract_address_const
    };
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::security::reentrancyguard::ReentrancyGuardComponent;
    
    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: ReentrancyGuardComponent, storage: reentrancy_guard, event: ReentrancyGuardEvent);
    
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;
    
    #[abi(embed_v0)]
    impl ReentrancyGuardImpl = ReentrancyGuardComponent::ReentrancyGuardImpl<ContractState>;
    impl ReentrancyGuardInternalImpl = ReentrancyGuardComponent::InternalImpl<ContractState>;
    
    #[storage]
    struct Storage {
        // Staking tokens
        staking_token: ContractAddress, // LP token address
        reward_token: ContractAddress,  // Reward token address
        
        // Reward parameters
        reward_rate: u256,              // Rewards per second
        reward_duration: u64,           // Duration of reward period
        period_finish: u64,             // When current reward period ends
        last_update_time: u64,          // Last time rewards were updated
        reward_per_token_stored: u256,  // Accumulated reward per token
        
        // User data
        user_reward_per_token_paid: LegacyMap<ContractAddress, u256>,
        rewards: LegacyMap<ContractAddress, u256>,
        balances: LegacyMap<ContractAddress, u256>,
        total_supply: u256,
        
        // Boosting mechanism
        boost_multiplier: LegacyMap<ContractAddress, u256>, // 1000 = 1x, 2000 = 2x
        boost_duration: LegacyMap<ContractAddress, u64>,
        boost_end_time: LegacyMap<ContractAddress, u64>,
        
        // Vesting
        vesting_duration: u64,
        user_vesting_schedules: LegacyMap<ContractAddress, VestingSchedule>,
        
        // Multi-token rewards
        additional_reward_tokens: LegacyMap<u256, ContractAddress>,
        additional_reward_rates: LegacyMap<u256, u256>,
        additional_rewards_count: u256,
        user_additional_rewards: LegacyMap<(ContractAddress, u256), u256>,
        additional_reward_per_token_stored: LegacyMap<u256, u256>,
        user_additional_reward_per_token_paid: LegacyMap<(ContractAddress, u256), u256>,
        
        // Emergency controls
        paused: bool,
        emergency_withdraw_enabled: bool,
        
        // Components
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        reentrancy_guard: ReentrancyGuardComponent::Storage,
    }
    
    #[derive(Drop, Copy, Serde, starknet::Store)]
    pub struct VestingSchedule {
        pub total_amount: u256,
        pub released_amount: u256,
        pub start_time: u64,
        pub duration: u64,
        pub cliff_duration: u64,
    }
    
    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        Staked: Staked,
        Withdrawn: Withdrawn,
        RewardPaid: RewardPaid,
        RewardAdded: RewardAdded,
        BoostActivated: BoostActivated,
        VestingScheduleCreated: VestingScheduleCreated,
        VestedTokensReleased: VestedTokensReleased,
        AdditionalRewardAdded: AdditionalRewardAdded,
        AdditionalRewardPaid: AdditionalRewardPaid,
        EmergencyWithdraw: EmergencyWithdraw,
        
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        ReentrancyGuardEvent: ReentrancyGuardComponent::Event,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Staked {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Withdrawn {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct RewardPaid {
        pub user: ContractAddress,
        pub reward: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct RewardAdded {
        pub reward: u256,
        pub duration: u64,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct BoostActivated {
        pub user: ContractAddress,
        pub multiplier: u256,
        pub duration: u64,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct VestingScheduleCreated {
        pub user: ContractAddress,
        pub amount: u256,
        pub duration: u64,
        pub cliff_duration: u64,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct VestedTokensReleased {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct AdditionalRewardAdded {
        pub token: ContractAddress,
        pub reward_rate: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct AdditionalRewardPaid {
        pub user: ContractAddress,
        pub token: ContractAddress,
        pub reward: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct EmergencyWithdraw {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    pub mod Errors {
        pub const ZERO_AMOUNT: felt252 = 'Amount cannot be zero';
        pub const INSUFFICIENT_BALANCE: felt252 = 'Insufficient balance';
        pub const REWARD_PERIOD_NOT_FINISHED: felt252 = 'Reward period not finished';
        pub const INVALID_DURATION: felt252 = 'Invalid duration';
        pub const CONTRACT_PAUSED: felt252 = 'Contract is paused';
        pub const EMERGENCY_WITHDRAW_DISABLED: felt252 = 'Emergency withdraw disabled';
        pub const BOOST_STILL_ACTIVE: felt252 = 'Boost still active';
        pub const CLIFF_NOT_REACHED: felt252 = 'Cliff period not reached';
        pub const NO_VESTED_TOKENS: felt252 = 'No vested tokens available';
    }
    
    #[constructor]
    fn constructor(
        ref self: ContractState,
        owner: ContractAddress,
        staking_token: ContractAddress,
        reward_token: ContractAddress,
        reward_duration: u64
    ) {
        self.ownable.initializer(owner);
        self.staking_token.write(staking_token);
        self.reward_token.write(reward_token);
        self.reward_duration.write(reward_duration);
        self.vesting_duration.write(86400 * 30); // 30 days default
    }
    
    #[abi(embed_v0)]
    impl YieldFarmingImpl of IYieldFarming<ContractState> {
        fn stake(ref self: ContractState, amount: u256) {
            self._check_not_paused();
            self.reentrancy_guard.start();
            assert(amount > 0, Errors::ZERO_AMOUNT);
            
            self._update_reward(get_caller_address());
            
            let staking_token = IERC20Dispatcher { contract_address: self.staking_token.read() };
            staking_token.transfer_from(get_caller_address(), get_contract_address(), amount);
            
            let user_balance = self.balances.read(get_caller_address());
            self.balances.write(get_caller_address(), user_balance + amount);
            self.total_supply.write(self.total_supply.read() + amount);
            
            self.emit(Staked { user: get_caller_address(), amount });
            self.reentrancy_guard.end();
        }
        
        fn withdraw(ref self: ContractState, amount: u256) {
            self._check_not_paused();
            self.reentrancy_guard.start();
            assert(amount > 0, Errors::ZERO_AMOUNT);
            
            let user_balance = self.balances.read(get_caller_address());
            assert(user_balance >= amount, Errors::INSUFFICIENT_BALANCE);
            
            self._update_reward(get_caller_address());
            
            self.balances.write(get_caller_address(), user_balance - amount);
            self.total_supply.write(self.total_supply.read() - amount);
            
            let staking_token = IERC20Dispatcher { contract_address: self.staking_token.read() };
            staking_token.transfer(get_caller_address(), amount);
            
            self.emit(Withdrawn { user: get_caller_address(), amount });
            self.reentrancy_guard.end();
        }
        
        fn claim_reward(ref self: ContractState) -> u256 {
            self._check_not_paused();
            self.reentrancy_guard.start();
            
            self._update_reward(get_caller_address());
            let reward = self.rewards.read(get_caller_address());
            
            if reward > 0 {
                self.rewards.write(get_caller_address(), 0);
                
                // Apply boost multiplier
                let boost_multiplier = self._get_current_boost_multiplier(get_caller_address());
                let boosted_reward = (reward * boost_multiplier) / 1000;
                
                // Create vesting schedule for rewards
                self._create_vesting_schedule(get_caller_address(), boosted_reward);
                
                self.emit(RewardPaid { user: get_caller_address(), reward: boosted_reward });
            }
            
            // Claim additional rewards
            self._claim_additional_rewards();
            
            self.reentrancy_guard.end();
            reward
        }
        
        fn claim_vested_rewards(ref self: ContractState) -> u256 {
            self.reentrancy_guard.start();
            
            let vesting_schedule = self.user_vesting_schedules.read(get_caller_address());
            let releasable_amount = self._calculate_releasable_amount(vesting_schedule);
            
            assert(releasable_amount > 0, Errors::NO_VESTED_TOKENS);
            
            let new_schedule = VestingSchedule {
                total_amount: vesting_schedule.total_amount,
                released_amount: vesting_schedule.released_amount + releasable_amount,
                start_time: vesting_schedule.start_time,
                duration: vesting_schedule.duration,
                cliff_duration: vesting_schedule.cliff_duration,
            };
            
            self.user_vesting_schedules.write(get_caller_address(), new_schedule);
            
            let reward_token = IERC20Dispatcher { contract_address: self.reward_token.read() };
            reward_token.transfer(get_caller_address(), releasable_amount);
            
            self.emit(VestedTokensReleased { 
                user: get_caller_address(), 
                amount: releasable_amount 
            });
            
            self.reentrancy_guard.end();
            releasable_amount
        }
        
        fn activate_boost(ref self: ContractState, multiplier: u256, duration: u64) {
            self.ownable.assert_only_owner();
            assert(multiplier >= 1000 && multiplier <= 5000, 'Invalid multiplier'); // 1x to 5x
            assert(duration > 0, Errors::INVALID_DURATION);
            
            let current_time = get_block_timestamp();
            self.boost_multiplier.write(get_caller_address(), multiplier);
            self.boost_duration.write(get_caller_address(), duration);
            self.boost_end_time.write(get_caller_address(), current_time + duration);
            
            self.emit(BoostActivated { 
                user: get_caller_address(), 
                multiplier, 
                duration 
            });
        }
        
        fn add_reward(ref self: ContractState, reward: u256) {
            self.ownable.assert_only_owner();
            self._update_reward(contract_address_const::<0>());
            
            let current_time = get_block_timestamp();
            let reward_duration = self.reward_duration.read();
            
            if current_time >= self.period_finish.read() {
                self.reward_rate.write(reward / reward_duration.into());
            } else {
                let remaining = self.period_finish.read() - current_time;
                let leftover = remaining.into() * self.reward_rate.read();
                self.reward_rate.write((reward + leftover) / reward_duration.into());
            }
            
            self.last_update_time.write(current_time);
            self.period_finish.write(current_time + reward_duration);
            
            let reward_token = IERC20Dispatcher { contract_address: self.reward_token.read() };
            reward_token.transfer_from(get_caller_address(), get_contract_address(), reward);
            
            self.emit(RewardAdded { reward, duration: reward_duration });
        }
        
        fn add_additional_reward_token(
            ref self: ContractState, 
            token: ContractAddress, 
            reward_rate: u256
        ) {
            self.ownable.assert_only_owner();
            let count = self.additional_rewards_count.read();
            
            self.additional_reward_tokens.write(count, token);
            self.additional_reward_rates.write(count, reward_rate);
            self.additional_rewards_count.write(count + 1);
            
            self.emit(AdditionalRewardAdded { token, reward_rate });
        }
        
        fn emergency_withdraw(ref self: ContractState) {
            assert(self.emergency_withdraw_enabled.read(), Errors::EMERGENCY_WITHDRAW_DISABLED);
            self.reentrancy_guard.start();
            
            let user_balance = self.balances.read(get_caller_address());
            assert(user_balance > 0, Errors::INSUFFICIENT_BALANCE);
            
            self.balances.write(get_caller_address(), 0);
            self.total_supply.write(self.total_supply.read() - user_balance);
            
            let staking_token = IERC20Dispatcher { contract_address: self.staking_token.read() };
            staking_token.transfer(get_caller_address(), user_balance);
            
            // Forfeit all rewards in emergency
            self.rewards.write(get_caller_address(), 0);
            
            self.emit(EmergencyWithdraw { user: get_caller_address(), amount: user_balance });
            self.reentrancy_guard.end();
        }
        
        // View functions
        fn balance_of(self: @ContractState, account: ContractAddress) -> u256 {
            self.balances.read(account)
        }
        
        fn total_supply(self: @ContractState) -> u256 {
            self.total_supply.read()
        }
        
        fn earned(self: @ContractState, account: ContractAddress) -> u256 {
            let balance = self.balances.read(account);
            let reward_per_token = self._reward_per_token();
            let user_reward_per_token_paid = self.user_reward_per_token_paid.read(account);
            
            let base_earned = (balance * (reward_per_token - user_reward_per_token_paid)) / 1000000000000000000;
            let current_rewards = self.rewards.read(account);
            
            base_earned + current_rewards
        }
        
        fn get_boost_info(self: @ContractState, user: ContractAddress) -> (u256, u64, u64) {
            (
                self.boost_multiplier.read(user),
                self.boost_duration.read(user),
                self.boost_end_time.read(user)
            )
        }
        
        fn get_vesting_info(self: @ContractState, user: ContractAddress) -> VestingSchedule {
            self.user_vesting_schedules.read(user)
        }
        
        fn get_releasable_amount(self: @ContractState, user: ContractAddress) -> u256 {
            let vesting_schedule = self.user_vesting_schedules.read(user);
            self._calculate_releasable_amount(vesting_schedule)
        }
        
        fn reward_per_token(self: @ContractState) -> u256 {
            self._reward_per_token()
        }
        
        fn last_time_reward_applicable(self: @ContractState) -> u64 {
            let current_time = get_block_timestamp();
            let period_finish = self.period_finish.read();
            
            if current_time < period_finish {
                current_time
            } else {
                period_finish
            }
        }
        
        // Admin functions
        fn set_paused(ref self: ContractState, paused: bool) {
            self.ownable.assert_only_owner();
            self.paused.write(paused);
        }
        
        fn set_emergency_withdraw(ref self: ContractState, enabled: bool) {
            self.ownable.assert_only_owner();
            self.emergency_withdraw_enabled.write(enabled);
        }
        
        fn set_vesting_duration(ref self: ContractState, duration: u64) {
            self.ownable.assert_only_owner();
            self.vesting_duration.write(duration);
        }
    }
    
    #[starknet::interface]
    pub trait IYieldFarming<TContractState> {
        fn stake(ref self: TContractState, amount: u256);
        fn withdraw(ref self: TContractState, amount: u256);
        fn claim_reward(ref self: TContractState) -> u256;
        fn claim_vested_rewards(ref self: TContractState) -> u256;
        fn activate_boost(ref self: TContractState, multiplier: u256, duration: u64);
        fn add_reward(ref self: TContractState, reward: u256);
        fn add_additional_reward_token(ref self: TContractState, token: ContractAddress, reward_rate: u256);
        fn emergency_withdraw(ref self: TContractState);
        
        // View functions
        fn balance_of(self: @TContractState, account: ContractAddress) -> u256;
        fn total_supply(self: @TContractState) -> u256;
        fn earned(self: @TContractState, account: ContractAddress) -> u256;
        fn get_boost_info(self: @TContractState, user: ContractAddress) -> (u256, u64, u64);
        fn get_vesting_info(self: @TContractState, user: ContractAddress) -> VestingSchedule;
        fn get_releasable_amount(self: @TContractState, user: ContractAddress) -> u256;
        fn reward_per_token(self: @TContractState) -> u256;
        fn last_time_reward_applicable(self: @TContractState) -> u64;
        
        // Admin functions
        fn set_paused(ref self: TContractState, paused: bool);
        fn set_emergency_withdraw(ref self: TContractState, enabled: bool);
        fn set_vesting_duration(ref self: TContractState, duration: u64);
    }
    
    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn _reward_per_token(self: @ContractState) -> u256 {
            let total_supply = self.total_supply.read();
            if total_supply == 0 {
                return self.reward_per_token_stored.read();
            }
            
            let last_time_applicable = self.last_time_reward_applicable();
            let last_update_time = self.last_update_time.read();
            let reward_rate = self.reward_rate.read();
            
            self.reward_per_token_stored.read() + 
                ((last_time_applicable - last_update_time).into() * reward_rate * 1000000000000000000) / total_supply
        }
        
        fn _update_reward(ref self: ContractState, account: ContractAddress) {
            let reward_per_token = self._reward_per_token();
            self.reward_per_token_stored.write(reward_per_token);
            self.last_update_time.write(self.last_time_reward_applicable());
            
            if !account.is_zero() {
                let balance = self.balances.read(account);
                let user_reward_per_token_paid = self.user_reward_per_token_paid.read(account);
                let earned = (balance * (reward_per_token - user_reward_per_token_paid)) / 1000000000000000000;
                
                self.rewards.write(account, self.rewards.read(account) + earned);
                self.user_reward_per_token_paid.write(account, reward_per_token);
            }
            
            // Update additional rewards
            self._update_additional_rewards(account);
        }
        
        fn _update_additional_rewards(ref self: ContractState, account: ContractAddress) {
            let count = self.additional_rewards_count.read();
            let mut i = 0;
            
            while i < count {
                let reward_rate = self.additional_reward_rates.read(i);
                let total_supply = self.total_supply.read();
                
                if total_supply > 0 {
                    let current_time = get_block_timestamp();
                    let last_update = self.last_update_time.read();
                    let time_diff = current_time - last_update;
                    
                    let reward_per_token_increment = (time_diff.into() * reward_rate * 1000000000000000000) / total_supply;
                    let current_reward_per_token = self.additional_reward_per_token_stored.read(i);
                    self.additional_reward_per_token_stored.write(i, current_reward_per_token + reward_per_token_increment);
                    
                    if !account.is_zero() {
                        let balance = self.balances.read(account);
                        let user_reward_per_token_paid = self.user_additional_reward_per_token_paid.read((account, i));
                        let earned = (balance * reward_per_token_increment) / 1000000000000000000;
                        
                        let current_rewards = self.user_additional_rewards.read((account, i));
                        self.user_additional_rewards.write((account, i), current_rewards + earned);
                        self.user_additional_reward_per_token_paid.write((account, i), current_reward_per_token + reward_per_token_increment);
                    }
                }
                
                i += 1;
            }
        }
        
        fn _claim_additional_rewards(ref self: ContractState) {
            let count = self.additional_rewards_count.read();
            let mut i = 0;
            
            while i < count {
                let reward = self.user_additional_rewards.read((get_caller_address(), i));
                if reward > 0 {
                    self.user_additional_rewards.write((get_caller_address(), i), 0);
                    
                    let token_address = self.additional_reward_tokens.read(i);
                    let token = IERC20Dispatcher { contract_address: token_address };
                    token.transfer(get_caller_address(), reward);
                    
                    self.emit(AdditionalRewardPaid { 
                        user: get_caller_address(), 
                        token: token_address, 
                        reward 
                    });
                }
                
                i += 1;
            }
        }
        
        fn _get_current_boost_multiplier(self: @ContractState, user: ContractAddress) -> u256 {
            let boost_end_time = self.boost_end_time.read(user);
            let current_time = get_block_timestamp();
            
            if current_time <= boost_end_time {
                self.boost_multiplier.read(user)
            } else {
                1000 // 1x multiplier (no boost)
            }
        }
        
        fn _create_vesting_schedule(ref self: ContractState, user: ContractAddress, amount: u256) {
            let current_time = get_block_timestamp();
            let vesting_duration = self.vesting_duration.read();
            let cliff_duration = vesting_duration / 4; // 25% cliff
            
            let existing_schedule = self.user_vesting_schedules.read(user);
            let new_schedule = VestingSchedule {
                total_amount: existing_schedule.total_amount + amount,
                released_amount: existing_schedule.released_amount,
                start_time: if existing_schedule.start_time == 0 { current_time } else { existing_schedule.start_time },
                duration: vesting_duration,
                cliff_duration,
            };
            
            self.user_vesting_schedules.write(user, new_schedule);
            
            self.emit(VestingScheduleCreated { 
                user, 
                amount, 
                duration: vesting_duration, 
                cliff_duration 
            });
        }
        
        fn _calculate_releasable_amount(self: @ContractState, schedule: VestingSchedule) -> u256 {
            if schedule.total_amount == 0 {
                return 0;
            }
            
            let current_time = get_block_timestamp();
            let cliff_time = schedule.start_time + schedule.cliff_duration;
            
            if current_time < cliff_time {
                return 0;
            }
            
            let vesting_end_time = schedule.start_time + schedule.duration;
            let vested_amount = if current_time >= vesting_end_time {
                schedule.total_amount
            } else {
                let time_since_start = current_time - schedule.start_time;
                (schedule.total_amount * time_since_start.into()) / schedule.duration.into()
            };
            
            if vested_amount > schedule.released_amount {
                vested_amount - schedule.released_amount
            } else {
                0
            }
        }
        
        fn _check_not_paused(self: @ContractState) {
            assert(!self.paused.read(), Errors::CONTRACT_PAUSED);
        }
    }
}
