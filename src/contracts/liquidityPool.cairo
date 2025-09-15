#[starknet::contract]
pub mod LiquidityPool {
    use starknet::{
        ContractAddress, get_caller_address, get_contract_address, get_block_timestamp,
        contract_address_const, get_tx_info
    };
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use openzeppelin::access::ownable::OwnableComponent;
    use openzeppelin::security::reentrancyguard::ReentrancyGuardComponent;
    use super::interfaces::ILiquidityPool;
    use super::libraries::AMMCalculations::{AMM, ReserveData, TWAPData, MEVProtectionData};
    
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
        // Core pool data
        token_a: ContractAddress,
        token_b: ContractAddress,
        reserve_a: u256,
        reserve_b: u256,
        block_timestamp_last: u64,
        
        // LP token data (ERC20 implementation)
        name: ByteArray,
        symbol: ByteArray,
        decimals: u8,
        total_supply: u256,
        balances: LegacyMap<ContractAddress, u256>,
        allowances: LegacyMap<(ContractAddress, ContractAddress), u256>,
        
        // Track individual liquidity positions
        user_liquidity_data: LegacyMap<ContractAddress, LiquidityPosition>,
        position_counter: u256,
        positions: LegacyMap<u256, LiquidityPosition>,
        user_positions: LegacyMap<ContractAddress, Array<u256>>,
        
        // Fee structure
        fee_rate: u256, // 30 = 0.3%
        protocol_fee_rate: u256, // 5 = 0.05%
        protocol_fee_a: u256,
        protocol_fee_b: u256,
        
        lp_reward_rate: u256,
        lp_reward_per_token_stored: u256,
        lp_user_reward_per_token_paid: LegacyMap<ContractAddress, u256>,
        lp_rewards: LegacyMap<ContractAddress, u256>,
        lp_last_update_time: u64,
        
        // Yield farming
        reward_token: ContractAddress,
        reward_rate: u256,
        last_update_time: u64,
        reward_per_token_stored: u256,
        user_reward_per_token_paid: LegacyMap<ContractAddress, u256>,
        rewards: LegacyMap<ContractAddress, u256>,
        staked_balances: LegacyMap<ContractAddress, u256>,
        total_staked: u256,
        
        // Impermanent Loss Protection
        il_protection_enabled: LegacyMap<ContractAddress, bool>,
        initial_deposit_value: LegacyMap<ContractAddress, u256>,
        initial_token_prices: LegacyMap<ContractAddress, (u256, u256)>,
        il_protection_fund: u256,
        
        user_il_positions: LegacyMap<ContractAddress, Array<ILPosition>>,
        il_position_counter: u256,
        il_positions: LegacyMap<u256, ILPosition>,
        il_protection_threshold: u256, // Minimum IL percentage to trigger protection (e.g., 500 = 5%)
        il_protection_coverage: u256,  // Coverage percentage (e.g., 8000 = 80% coverage)
        il_protection_duration: u64,   // Protection duration in seconds
        oracle_address: ContractAddress, // Price oracle for accurate IL calculation
        
        // Enhanced MEV Protection
        last_trade_block: LegacyMap<ContractAddress, u64>,
        trade_cooldown: u64,
        max_trade_size: u256,
        
        // TWAP and price tracking
        twap_data: TWAPData,
        mev_protection: MEVProtectionData,
        
        // Advanced MEV protection
        user_trade_count: LegacyMap<ContractAddress, u256>,
        user_last_trade_block: LegacyMap<ContractAddress, u64>,
        block_trade_volume: LegacyMap<u64, u256>,
        suspicious_addresses: LegacyMap<ContractAddress, bool>,
        
        // Commit-reveal scheme
        commitments: LegacyMap<ContractAddress, u256>,
        commitment_blocks: LegacyMap<ContractAddress, u64>,
        reveal_window: u64,
        
        // Dynamic fees
        base_fee: u256,
        max_fee: u256,
        volatility_factor: u256,
        volume_factor: u256,
        
        // Batch auction
        batch_auction_enabled: bool,
        batch_duration: u64,
        current_batch_end: u64,
        buy_orders: LegacyMap<u64, Array<(ContractAddress, u256, u256)>>, // batch_id -> orders
        sell_orders: LegacyMap<u64, Array<(ContractAddress, u256, u256)>>,
        batch_counter: u64,
        
        // Circuit breaker
        circuit_breaker_enabled: bool,
        price_change_threshold: u256,
        volume_spike_threshold: u256,
        circuit_breaker_duration: u64,
        circuit_breaker_end_time: u64,
        
        // Components
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        reentrancy_guard: ReentrancyGuardComponent::Storage,
    }
    
    #[derive(Drop, Copy, Serde, starknet::Store)]
    pub struct LiquidityPosition {
        pub id: u256,
        pub owner: ContractAddress,
        pub liquidity: u256,
        pub token_a_amount: u256,
        pub token_b_amount: u256,
        pub timestamp: u64,
        pub last_fee_growth_a: u256,
        pub last_fee_growth_b: u256,
        pub unclaimed_fees_a: u256,
        pub unclaimed_fees_b: u256,
    }
    
    #[derive(Drop, Copy, Serde, starknet::Store)]
    pub struct ILPosition {
        pub id: u256,
        pub user: ContractAddress,
        pub liquidity_amount: u256,
        pub initial_token_a_amount: u256,
        pub initial_token_b_amount: u256,
        pub initial_token_a_price: u256,
        pub initial_token_b_price: u256,
        pub initial_total_value: u256,
        pub deposit_timestamp: u64,
        pub protection_end_timestamp: u64,
        pub is_active: bool,
        pub compensation_claimed: u256,
    }
    
    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        // ERC20 events
        Transfer: Transfer,
        Approval: Approval,
        
        // Liquidity events
        Mint: Mint,
        Burn: Burn,
        Swap: Swap,
        Sync: Sync,
        
        PositionCreated: PositionCreated,
        PositionUpdated: PositionUpdated,
        FeesCollected: FeesCollected,
        LPRewardPaid: LPRewardPaid,
        
        // Yield farming events
        Staked: Staked,
        Unstaked: Unstaked,
        RewardPaid: RewardPaid,
        
        // IL Protection events
        ILProtectionEnabled: ILProtectionEnabled,
        ILCompensationPaid: ILCompensationPaid,
        
        // MEV Protection events
        MEVDetected: MEVDetected,
        SuspiciousActivity: SuspiciousActivity,
        CircuitBreakerTriggered: CircuitBreakerTriggered,
        CommitmentMade: CommitmentMade,
        TradeRevealed: TradeRevealed,
        BatchAuctionExecuted: BatchAuctionExecuted,
        
        // Component events
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        ReentrancyGuardEvent: ReentrancyGuardComponent::Event,
    }
    #[derive(Drop, starknet::Event)]
    pub struct Transfer {
        #[key]
        pub from: ContractAddress,
        #[key]
        pub to: ContractAddress,
        pub value: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Approval {
        #[key]
        pub owner: ContractAddress,
        #[key]
        pub spender: ContractAddress,
        pub value: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Mint {
        pub sender: ContractAddress,
        pub amount_a: u256,
        pub amount_b: u256,
        pub liquidity: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Burn {
        pub sender: ContractAddress,
        pub amount_a: u256,
        pub amount_b: u256,
        pub liquidity: u256,
        pub to: ContractAddress,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Swap {
        pub sender: ContractAddress,
        pub amount_in: u256,
        pub amount_out: u256,
        pub token_in: ContractAddress,
        pub token_out: ContractAddress,
        pub to: ContractAddress,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Sync {
        pub reserve_a: u256,
        pub reserve_b: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Staked {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct Unstaked {
        pub user: ContractAddress,
        pub amount: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct RewardPaid {
        pub user: ContractAddress,
        pub reward: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct ILProtectionEnabled {
        pub user: ContractAddress,
        pub initial_value: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct ILCompensationPaid {
        pub user: ContractAddress,
        pub compensation: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct PositionCreated {
        pub position_id: u256,
        pub owner: ContractAddress,
        pub liquidity: u256,
        pub amount_a: u256,
        pub amount_b: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct PositionUpdated {
        pub position_id: u256,
        pub liquidity_delta: u256,
        pub amount_a_delta: u256,
        pub amount_b_delta: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct FeesCollected {
        pub position_id: u256,
        pub owner: ContractAddress,
        pub fees_a: u256,
        pub fees_b: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct LPRewardPaid {
        pub user: ContractAddress,
        pub reward: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct MEVDetected {
        pub user: ContractAddress,
        pub detection_type: felt252,
        pub severity: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct SuspiciousActivity {
        pub user: ContractAddress,
        pub activity_type: felt252,
        pub block_number: u64,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct CircuitBreakerTriggered {
        pub trigger_type: felt252,
        pub duration: u64,
        pub trigger_value: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct CommitmentMade {
        pub user: ContractAddress,
        pub commitment: u256,
        pub block_number: u64,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct TradeRevealed {
        pub user: ContractAddress,
        pub amount: u256,
        pub nonce: u256,
    }
    
    #[derive(Drop, starknet::Event)]
    pub struct BatchAuctionExecuted {
        pub batch_id: u64,
        pub clearing_price: u256,
        pub volume: u256,
        pub participants: u256,
    }
    
    pub mod Errors {
        pub const INSUFFICIENT_LIQUIDITY: felt252 = 'Insufficient liquidity';
        pub const INSUFFICIENT_INPUT_AMOUNT: felt252 = 'Insufficient input amount';
        pub const INSUFFICIENT_OUTPUT_AMOUNT: felt252 = 'Insufficient output amount';
        pub const EXPIRED: felt252 = 'Transaction expired';
        pub const IDENTICAL_ADDRESSES: felt252 = 'Identical addresses';
        pub const ZERO_ADDRESS: felt252 = 'Zero address';
        pub const INSUFFICIENT_A_AMOUNT: felt252 = 'Insufficient A amount';
        pub const INSUFFICIENT_B_AMOUNT: felt252 = 'Insufficient B amount';
        pub const INSUFFICIENT_LIQUIDITY_MINTED: felt252 = 'Insufficient liquidity minted';
        pub const INSUFFICIENT_LIQUIDITY_BURNED: felt252 = 'Insufficient liquidity burned';
        pub const TRADE_COOLDOWN: felt252 = 'Trade in cooldown period';
        pub const TRADE_SIZE_EXCEEDED: felt252 = 'Trade size exceeded';
        pub const MEV_DETECTED: felt252 = 'MEV attack detected';
        pub const PRICE_IMPACT_TOO_HIGH: felt252 = 'Price impact too high';
        pub const CIRCUIT_BREAKER_ACTIVE: felt252 = 'Circuit breaker active';
        pub const SUSPICIOUS_ADDRESS: felt252 = 'Suspicious address blocked';
        pub const INVALID_COMMITMENT: felt252 = 'Invalid commitment';
        pub const REVEAL_WINDOW_EXPIRED: felt252 = 'Reveal window expired';
        pub const BATCH_AUCTION_ACTIVE: felt252 = 'Batch auction in progress';
    }
    
    #[constructor]
    fn constructor(
        ref self: ContractState, 
        owner: ContractAddress,
        name: ByteArray,
        symbol: ByteArray
    ) {
        self.ownable.initializer(owner);
        self.name.write(name);
        self.symbol.write(symbol);
        self.decimals.write(18);
        self.trade_cooldown.write(1); // 1 second cooldown
        self.max_trade_size.write(1000000 * 1000000000000000000); // 1M tokens max
        self.position_counter.write(0);
        
        // Initialize MEV protection
        let mev_protection = MEVProtectionData {
            max_price_impact: 1000, // 10% max price impact
            sandwich_protection_window: 3, // 3 blocks
            volume_threshold: 100000 * 1000000000000000000, // 100k tokens
            consecutive_trade_limit: 5,
            flash_loan_protection: true,
        };
        self.mev_protection.write(mev_protection);
    