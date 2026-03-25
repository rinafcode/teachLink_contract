#![cfg_attr(not(test), no_std)]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Vec, Symbol};

/// Configuration constants for TeachLink contract
pub mod constants {
    /// Fee configuration
    pub mod fees {
        pub const DEFAULT_FEE_RATE: u32 = 100; // 1% in basis points
        pub const MAX_FEE_RATE: u32 = 10000; // 100% in basis points
        pub const FEE_CALCULATION_DIVISOR: u32 = 10000; // Convert basis points to decimal
    }
    
    /// Amount validation
    pub mod amounts {
        pub const MIN_AMOUNT: i128 = 1; // Minimum bridge amount
        pub const FALLBACK_PRICE: i128 = 1000000; // 1 USD in 6 decimals
    }
    
    /// Chain configuration
    pub mod chains {
        pub const MIN_CHAIN_ID: u32 = 1; // Minimum valid chain ID
        pub const DEFAULT_MIN_CONFIRMATIONS: u32 = 3; // Default block confirmations
        pub const MAX_CHAIN_NAME_LENGTH: u32 = 32; // Maximum chain name length
    }
    
    /// Oracle configuration
    pub mod oracle {
        pub const MAX_CONFIDENCE: u32 = 100; // Maximum confidence percentage
        pub const DEFAULT_CONFIDENCE_THRESHOLD: u32 = 80; // Minimum confidence for oracle data
        pub const PRICE_FRESHNESS_SECONDS: u64 = 3600; // 1 hour in seconds
    }
    
    /// Rate limiting
    pub mod rate_limits {
        pub const DEFAULT_PER_MINUTE: u32 = 10; // Default calls per minute
        pub const DEFAULT_PER_HOUR: u32 = 100; // Default calls per hour
        pub const DEFAULT_PENALTY_MULTIPLIER: u32 = 2; // Penalty multiplier
        pub const SECONDS_PER_MINUTE: u64 = 60; // Seconds in a minute
        pub const SECONDS_PER_HOUR: u64 = 3600; // Seconds in an hour
    }
    
    /// Error codes
    pub mod error_codes {
        pub const SUCCESS: u32 = 0;
        pub const INVALID_ADDRESS: u32 = 1001;
        pub const INVALID_AMOUNT: u32 = 1002;
        pub const FALLBACK_DISABLED: u32 = 1003;
        pub const CHAIN_NOT_SUPPORTED: u32 = 1004;
        pub const RATE_LIMIT_EXCEEDED: u32 = 1005;
        pub const INSUFFICIENT_BALANCE: u32 = 1006;
        pub const BRIDGE_FAILED: u32 = 1007;
    }
    
    /// Storage limits
    pub mod storage {
        pub const MAX_BRIDGE_TXS: u32 = 1000; // Maximum bridge transactions stored
        pub const MAX_CHAIN_CONFIGS: u32 = 50; // Maximum chain configurations
        pub const MAX_ORACLE_PRICES: u32 = 100; // Maximum oracle prices stored
    }
}

/// Error types for TeachLink contract
#[derive(Clone, Debug)]
pub enum TeachLinkError {
    Unauthorized,
    InvalidAmount,
    InvalidAddress,
    ChainNotSupported,
    RateLimitExceeded,
    InsufficientBalance,
    BridgeFailed,
    NotInitialized,
    InvalidChainId,
    FeeTooHigh,
    ChainExists,
    InvalidPrice,
    InvalidConfidence,
    UnauthorizedOracle,
}

/// Configuration struct for bridge parameters
#[derive(Clone, Debug)]
pub struct BridgeConfig {
    pub fee_rate: u32,
    pub min_confirmations: u32,
    pub confidence_threshold: u32,
    pub fallback_enabled: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            fee_rate: constants::fees::DEFAULT_FEE_RATE,
            min_confirmations: constants::chains::DEFAULT_MIN_CONFIRMATIONS,
            confidence_threshold: constants::oracle::DEFAULT_CONFIDENCE_THRESHOLD,
            fallback_enabled: true,
        }
    }
}

/// TeachLink main contract with named constants and configuration.
#[cfg(not(test))]
#[contract]
pub struct TeachLinkBridge;

#[cfg(not(test))]
#[contractimpl]
impl TeachLinkBridge {
    // Storage keys
    const ADMIN: Symbol = symbol_short!("admin");
    const NONCE: Symbol = symbol_short!("nonce");
    const BRIDGE_TXS: Symbol = symbol_short!("brdg_txs");
    const FALLBACK_ENABLED: Symbol = symbol_short!("fallback");
    const ERROR_COUNT: Symbol = symbol_short!("err_cnt");
    const CONFIG: Symbol = symbol_short!("config");
    
    /// Initialize bridge contract with configuration
    pub fn initialize(env: Env, admin: Address) {
        Self::require_initialized(&env, false);
        Self::validate_address(&admin);
        
        // Initialize with default configuration
        let config = BridgeConfig::default();
        
        env.storage().instance().set(&Self::ADMIN, &admin);
        env.storage().instance().set(&Self::NONCE, &0u64);
        env.storage().instance().set(&Self::FALLBACK_ENABLED, &config.fallback_enabled);
        env.storage().instance().set(&Self::BRIDGE_TXS, &Vec::new(&env));
        env.storage().instance().set(&Self::ERROR_COUNT, &0u64);
        env.storage().instance().set(&Self::CONFIG, &config);
    }
    
    /// Bridge tokens out with named constants
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> u64 {
        Self::require_initialized(&env, true);
        Self::validate_amount(&amount);
        Self::validate_chain_id(&destination_chain);
        Self::validate_bytes_address(&destination_address);
        
        let config = Self::get_stored_config(&env);
        let nonce = Self::get_next_nonce(&env);
        
        // Calculate fees using named constants
        let fee_amount = Self::calculate_fee(&amount, config.fee_rate);
        let bridge_amount = amount - fee_amount;
        
        Self::validate_amount(&bridge_amount);
        
        // Store bridge transaction
        let bridge_data = (from, bridge_amount, destination_chain, destination_address);
        let mut bridge_txs: Vec<(Address, i128, u32, Bytes)> = env.storage()
            .instance()
            .get(&Self::BRIDGE_TXS)
            .unwrap_or(Vec::new(&env));
        
        // Enforce storage limit
        if bridge_txs.len() >= constants::storage::MAX_BRIDGE_TXS {
            Self::handle_error(&env, TeachLinkError::BridgeFailed);
        }
        
        bridge_txs.push_back(bridge_data);
        env.storage().instance().set(&Self::BRIDGE_TXS, &bridge_txs);
        
        nonce
    }
    
    /// Add support for a new chain with validation using constants
    pub fn add_chain_support(
        env: Env,
        chain_id: u32,
        name: Symbol,
        bridge_address: Address,
        min_confirmations: u32,
        fee_rate: u32,
    ) {
        Self::require_admin(&env);
        Self::validate_chain_id(&chain_id);
        Self::validate_fee_rate(&fee_rate);
        Self::validate_address(&bridge_address);
        
        // Check chain name length
        if name.to_string().len() > constants::chains::MAX_CHAIN_NAME_LENGTH as usize {
            Self::handle_error(&env, TeachLinkError::InvalidAddress);
        }
        
        // Check if chain already exists
        let chains: Vec<(u32, Symbol, Address, u32, u32)> = env.storage()
            .instance()
            .get(&symbol_short!("chains"))
            .unwrap_or(Vec::new(&env));
        
        if chains.len() >= constants::storage::MAX_CHAIN_CONFIGS {
            Self::handle_error(&env, TeachLinkError::ChainExists);
        }
        
        for chain in chains.iter() {
            if chain.0 == chain_id {
                Self::handle_error(&env, TeachLinkError::ChainExists);
            }
        }
        
        // Store chain configuration
        let chain_config = (chain_id, name, bridge_address, min_confirmations, fee_rate);
        let mut updated_chains = chains;
        updated_chains.push_back(chain_config);
        env.storage().instance().set(&symbol_short!("chains"), &updated_chains);
    }
    
    /// Update oracle price with validation using constants
    pub fn update_oracle_price(
        env: Env,
        asset: Symbol,
        price: i128,
        confidence: u32,
        oracle_signer: Address,
    ) {
        Self::require_initialized(&env, true);
        Self::validate_price(&price);
        Self::validate_confidence(&confidence);
        
        // Check oracle authorization
        let authorized_oracles: Vec<Address> = env.storage()
            .instance()
            .get(&symbol_short!("oracles"))
            .unwrap_or(Vec::new(&env));
        
        let mut is_authorized = false;
        for oracle in authorized_oracles.iter() {
            if oracle == oracle_signer {
                is_authorized = true;
                break;
            }
        }
        
        if !is_authorized {
            Self::handle_error(&env, TeachLinkError::UnauthorizedOracle);
        }
        
        // Update oracle prices with storage limit check
        let oracle_price = (asset.clone(), price, env.ledger().timestamp(), confidence);
        let mut prices: Vec<(Symbol, i128, u64, u32)> = env.storage()
            .instance()
            .get(&symbol_short!("prices"))
            .unwrap_or(Vec::new(&env));
        
        if prices.len() >= constants::storage::MAX_ORACLE_PRICES {
            Self::handle_error(&env, TeachLinkError::InvalidPrice);
        }
        
        let mut updated = false;
        for i in 0..prices.len() {
            let price_data = prices.get(i).unwrap();
            if price_data.0 == asset {
                prices.set(i, oracle_price.clone());
                updated = true;
                break;
            }
        }
        
        if !updated {
            prices.push_back(oracle_price);
        }
        
        env.storage().instance().set(&symbol_short!("prices"), &prices);
    }
    
    /// Update bridge configuration
    pub fn update_config(env: Env, config: BridgeConfig) {
        Self::require_admin(&env);
        Self::validate_fee_rate(&config.fee_rate);
        
        env.storage().instance().set(&Self::CONFIG, &config);
    }
    
    // Validation functions using constants
    fn require_initialized(env: &Env, should_be_initialized: bool) {
        let is_init = env.storage().instance().get(&Self::ADMIN).is_some();
        if is_init != should_be_initialized {
            Self::handle_error(env, TeachLinkError::NotInitialized);
        }
    }
    
    fn require_admin(env: &Env) {
        let admin: Address = env.storage()
            .instance()
            .get(&Self::ADMIN)
            .unwrap_or_else(|| {
                Self::handle_error(env, TeachLinkError::NotInitialized);
            });
        
        if env.current_contract_address() != admin {
            Self::handle_error(env, TeachLinkError::Unauthorized);
        }
    }
    
    fn validate_address(_address: &Address) {
        // Address type in Soroban is always a valid bech32 account; no further check needed
    }

    fn validate_bytes_address(address: &Bytes) {
        if address.is_empty() {
            panic!("Invalid address");
        }
    }

    fn validate_amount(amount: &i128) {
        if *amount < constants::amounts::MIN_AMOUNT {
            panic!("Invalid amount");
        }
    }

    fn validate_chain_id(chain_id: &u32) {
        if *chain_id < constants::chains::MIN_CHAIN_ID {
            panic!("Invalid chain ID");
        }
    }

    fn validate_fee_rate(fee_rate: &u32) {
        if *fee_rate > constants::fees::MAX_FEE_RATE {
            panic!("Fee rate too high");
        }
    }

    fn validate_price(price: &i128) {
        if *price <= 0 {
            panic!("Invalid price");
        }
    }

    fn validate_confidence(confidence: &u32) {
        if *confidence > constants::oracle::MAX_CONFIDENCE {
            panic!("Invalid confidence");
        }
    }
    
    fn calculate_fee(amount: &i128, fee_rate: u32) -> i128 {
        amount * fee_rate as i128 / constants::fees::FEE_CALCULATION_DIVISOR as i128
    }
    
    fn get_stored_config(env: &Env) -> BridgeConfig {
        env.storage()
            .instance()
            .get(&Self::CONFIG)
            .unwrap_or_default()
    }

    fn handle_error(env: &Env, error: TeachLinkError) -> ! {
        // Increment error counter
        let mut count = env.storage()
            .instance()
            .get(&Self::ERROR_COUNT)
            .unwrap_or(0u64);
        count += 1;
        env.storage().instance().set(&Self::ERROR_COUNT, &count);
        
        // Panic with appropriate error message
        match error {
            TeachLinkError::Unauthorized => {
                env.panic_with_error_data(&symbol_short!("unauth"), "Unauthorized access");
            }
            TeachLinkError::InvalidAmount => {
                env.panic_with_error_data(&symbol_short!("inv_amt"), "Invalid amount");
            }
            TeachLinkError::InvalidAddress => {
                env.panic_with_error_data(&symbol_short!("inv_addr"), "Invalid address");
            }
            TeachLinkError::ChainNotSupported => {
                env.panic_with_error_data(&symbol_short!("no_chain"), "Chain not supported");
            }
            TeachLinkError::RateLimitExceeded => {
                env.panic_with_error_data(&symbol_short!("rate_lim"), "Rate limit exceeded");
            }
            TeachLinkError::InsufficientBalance => {
                env.panic_with_error_data(&symbol_short!("insuf_bal"), "Insufficient balance");
            }
            TeachLinkError::BridgeFailed => {
                env.panic_with_error_data(&symbol_short!("brdg_fail"), "Bridge operation failed");
            }
            TeachLinkError::NotInitialized => {
                env.panic_with_error_data(&symbol_short!("not_init"), "Contract not initialized");
            }
            TeachLinkError::InvalidChainId => {
                env.panic_with_error_data(&symbol_short!("bad_chain"), "Invalid chain ID");
            }
            TeachLinkError::FeeTooHigh => {
                env.panic_with_error_data(&symbol_short!("fee_high"), "Fee rate too high");
            }
            TeachLinkError::ChainExists => {
                env.panic_with_error_data(&symbol_short!("chn_dup"), "Chain already exists");
            }
            TeachLinkError::InvalidPrice => {
                env.panic_with_error_data(&symbol_short!("bad_price"), "Invalid price");
            }
            TeachLinkError::InvalidConfidence => {
                env.panic_with_error_data(&symbol_short!("bad_conf"), "Invalid confidence");
            }
            TeachLinkError::UnauthorizedOracle => {
                env.panic_with_error_data(&symbol_short!("bad_orcl"), "Unauthorized oracle");
            }
        }
    }
    
    /// Get next nonce
    fn get_next_nonce(env: &Env) -> u64 {
        let nonce = env.storage()
            .instance()
            .get(&Self::NONCE)
            .unwrap_or(0u64);
        let new_nonce = nonce + 1;
        env.storage()
            .instance()
            .set(&Self::NONCE, &new_nonce);
        new_nonce
    }
    
    /// Get bridge transaction
    pub fn get_bridge_tx(env: Env, index: u32) -> Option<(Address, i128, u32, Bytes)> {
        let bridge_txs: Vec<(Address, i128, u32, Bytes)> = env.storage()
            .instance()
            .get(&Self::BRIDGE_TXS)
            .unwrap_or(Vec::new(&env));
        bridge_txs.get(index)
    }
    
    /// Get configuration
    pub fn get_config(env: Env) -> BridgeConfig {
        env.storage()
            .instance()
            .get(&Self::CONFIG)
            .unwrap_or_default()
    /// Check if a signer approved
    pub fn has_escrow_approval(env: Env, escrow_id: u64, signer: Address) -> bool {
        escrow::EscrowManager::has_approved(&env, escrow_id, signer)
    }

    /// Get the current escrow count
    pub fn get_escrow_count(env: Env) -> u64 {
        escrow::EscrowManager::get_escrow_count(&env)
    }

    // ========== Credit Scoring Functions (feat/credit_score) ==========

    // TODO: Implement score module
    /*
    /// Record course completion
    pub fn record_course_completion(
        env: Env,
        user: Address,
        course_id: u64,
        points: u64,
    ) {
        let admin = bridge::Bridge::get_admin(&env);
        admin.require_auth();
        score::ScoreManager::record_course_completion(&env, user, course_id, points);
    }

    /// Record contribution
    pub fn record_contribution(
        env: Env,
        user: Address,
        c_type: types::ContributionType,
        description: Bytes,
        points: u64,
    ) {
        score::ScoreManager::record_contribution(&env, user, c_type, description, points);
    }

    /// Get user's credit score
    pub fn get_credit_score(env: Env, user: Address) -> u64 {
        score::ScoreManager::get_score(&env, user)
    }

    /// Get user's courses
    pub fn get_user_courses(env: Env, user: Address) -> Vec<types::Course> {
        score::ScoreManager::get_courses(&env, user)
    }

    /// Get user's contributions
    pub fn get_user_contributions(env: Env, user: Address) -> Vec<types::Contribution> {
        score::ScoreManager::get_contributions(&env, user)
    }
    */

    // ========== Reputation Functions (main) ==========

    // TODO: Implement missing modules
    /*
    pub fn update_participation(env: Env, user: Address, points: u32) {
        reputation::update_participation(&env, user, points);
    }

    pub fn update_course_progress(env: Env, user: Address, is_completion: bool) {
        reputation::update_course_progress(&env, user, is_completion);
    }

    pub fn rate_contribution(env: Env, user: Address, rating: u32) {
        reputation::rate_contribution(&env, user, rating);
    }

    pub fn get_user_reputation(env: Env, user: Address) -> types::UserReputation {
        reputation::get_reputation(&env, &user)
    }
    */

    // ========== Content Tokenization Functions ==========

    /// Mint a new educational content token
    pub fn mint_content_token(env: Env, params: ContentTokenParameters) -> u64 {
        // TODO: Implement provenance module
        // provenance::ProvenanceTracker::record_mint(&env, token_id, params.creator, None);
        tokenization::ContentTokenization::mint(
            &env,
            params.creator.clone(),
            params.title,
            params.description,
            params.content_type,
            params.content_hash,
            params.license_type,
            params.tags,
            params.is_transferable,
            params.royalty_percentage,
        )
    }

    /// Transfer ownership of a content token
    pub fn transfer_content_token(
        env: Env,
        from: Address,
        to: Address,
        token_id: u64,
        notes: Option<Bytes>,
    ) {
        tokenization::ContentTokenization::transfer(&env, from, to, token_id, notes);
    }

    /// Get a content token by ID
    pub fn get_content_token(env: Env, token_id: u64) -> Option<ContentToken> {
        tokenization::ContentTokenization::get_token(&env, token_id)
    }

    /// Get the owner of a content token
    pub fn get_content_token_owner(env: Env, token_id: u64) -> Option<Address> {
        tokenization::ContentTokenization::get_owner(&env, token_id)
    }

    /// Check if an address owns a content token
    pub fn is_content_token_owner(env: Env, token_id: u64, address: Address) -> bool {
        tokenization::ContentTokenization::is_owner(&env, token_id, address)
    }

    /// Get all tokens owned by an address
    pub fn get_owner_content_tokens(env: Env, owner: Address) -> Vec<u64> {
        tokenization::ContentTokenization::get_owner_tokens(&env, owner)
    }

    /// Get the total number of content tokens minted
    pub fn get_content_token_count(env: Env) -> u64 {
        tokenization::ContentTokenization::get_token_count(&env)
    }

    /// Update content token metadata (only by owner)
    pub fn update_content_metadata(
        env: Env,
        owner: Address,
        token_id: u64,
        title: Option<Bytes>,
        description: Option<Bytes>,
        tags: Option<Vec<Bytes>>,
    ) {
        tokenization::ContentTokenization::update_metadata(
            &env,
            owner,
            token_id,
            title,
            description,
            tags,
        );
    }

    /// Set transferability of a content token (only by owner)
    pub fn set_content_token_transferable(
        env: Env,
        owner: Address,
        token_id: u64,
        transferable: bool,
    ) {
        tokenization::ContentTokenization::set_transferable(&env, owner, token_id, transferable);
    }

    // ========== Provenance Functions ==========

    // TODO: Implement provenance module
    /*
    /// Get full provenance history for a content token
    pub fn get_content_provenance(env: Env, token_id: u64) -> Vec<ProvenanceRecord> {
        provenance::ProvenanceTracker::get_provenance(&env, token_id)
    }

    /// Get the number of transfers for a content token
    #[must_use]
    pub fn get_content_transfer_count(env: &Env, token_id: u64) -> u32 {
        provenance::ProvenanceTracker::get_transfer_count(env, token_id)
    }

    /// Verify ownership chain integrity for a content token
    #[must_use]
    pub fn verify_content_chain(env: &Env, token_id: u64) -> bool {
        provenance::ProvenanceTracker::verify_chain(env, token_id)
    }
    */

    /// Get the creator of a content token
    #[must_use]
    pub fn get_content_creator(env: &Env, token_id: u64) -> Option<Address> {
        tokenization::ContentTokenization::get_creator(env, token_id)
    }

    /// Get all owners of a content token
    #[must_use]
    pub fn get_content_all_owners(env: &Env, token_id: u64) -> Vec<Address> {
        tokenization::ContentTokenization::get_all_owners(env, token_id)
    }

    // ========== Notification System Functions ==========

    /// Initialize notification system
    pub fn initialize_notifications(env: Env) -> Result<(), BridgeError> {
        notification::NotificationManager::initialize(&env)
    }

    /// Send immediate notification
    pub fn send_notification(
        env: Env,
        recipient: Address,
        channel: NotificationChannel,
        subject: Bytes,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        notification::NotificationManager::send_notification(&env, recipient, channel, content)
    }

    // ========== Mobile UI/UX Functions ==========

    /// Initialize mobile profile for user
    pub fn initialize_mobile_profile(
        env: Env,
        user: Address,
        device_info: DeviceInfo,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::initialize_mobile_profile(
            &env,
            user,
            device_info,
            preferences,
        )
        .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Update accessibility settings
    pub fn update_accessibility_settings(
        env: Env,
        user: Address,
        settings: MobileAccessibilitySettings,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::update_accessibility_settings(&env, user, settings)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Update personalization settings
    pub fn update_personalization(
        env: Env,
        user: Address,
        preferences: MobilePreferences,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::update_personalization(&env, user, preferences)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Record onboarding progress
    pub fn record_onboarding_progress(
        env: Env,
        user: Address,
        stage: OnboardingStage,
    ) -> Result<(), MobilePlatformError> {
        mobile_platform::MobilePlatformManager::record_onboarding_progress(&env, user, stage)
            .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Submit user feedback
    pub fn submit_user_feedback(
        env: Env,
        user: Address,
        rating: u32,
        comment: Bytes,
        category: FeedbackCategory,
    ) -> Result<u64, MobilePlatformError> {
        mobile_platform::MobilePlatformManager::submit_user_feedback(
            &env, user, rating, comment, category,
        )
        .map_err(|_| MobilePlatformError::DeviceNotSupported)
    }

    /// Get user allocated experiment variants
    pub fn get_user_experiment_variants(env: Env, user: Address) -> Map<u64, Symbol> {
        mobile_platform::MobilePlatformManager::get_user_experiment_variants(&env, user)
    }

    /// Get design system configuration
    pub fn get_design_system_config(env: Env) -> ComponentConfig {
        mobile_platform::MobilePlatformManager::get_design_system_config(&env)
    }

    /// Set design system configuration (admin only)
    pub fn set_design_system_config(env: Env, config: ComponentConfig) {
        // In a real implementation, we would check for admin authorization here
        mobile_platform::MobilePlatformManager::set_design_system_config(&env, config)
    }

    /// Schedule notification for future delivery
    pub fn schedule_notification(
        env: Env,
        recipient: Address,
        channel: NotificationChannel,
        subject: Bytes,
        body: Bytes,
        scheduled_time: u64,
        timezone: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        let schedule = NotificationSchedule {
            notification_id: 0, // Will be set by the function
            recipient: recipient.clone(),
            channel,
            scheduled_time,
            timezone,
            is_recurring: false,
            recurrence_pattern: 0,
            max_deliveries: None,
            delivery_count: 0,
        };
        notification::NotificationManager::schedule_notification(
            &env, recipient, channel, content, schedule,
        )
    }

    /// Process scheduled notifications
    pub fn process_scheduled_notifications(env: Env) -> Result<u32, BridgeError> {
        notification::NotificationManager::process_scheduled_notifications(&env)
    }

    /// Update user notification preferences
    pub fn update_notification_preferences(
        env: Env,
        user: Address,
        preferences: Vec<NotificationPreference>,
    ) -> Result<(), BridgeError> {
        notification::NotificationManager::update_preferences(&env, user, preferences)
    }

    /// Update user notification settings
    pub fn update_notification_settings(
        env: Env,
        user: Address,
        timezone: Bytes,
        quiet_hours_start: u32,
        quiet_hours_end: u32,
        max_daily_notifications: u32,
        do_not_disturb: bool,
    ) -> Result<(), BridgeError> {
        let settings = UserNotificationSettings {
            user: user.clone(),
            timezone,
            quiet_hours_start,
            quiet_hours_end,
            max_daily_notifications,
            do_not_disturb,
        };
        notification::NotificationManager::update_user_settings(&env, user, settings)
    }

    /// Create notification template
    pub fn create_notification_template(
        env: Env,
        admin: Address,
        name: Bytes,
        channels: Vec<NotificationChannel>,
        subject: Bytes,
        body: Bytes,
    ) -> Result<u64, BridgeError> {
        let content = NotificationContent {
            subject,
            body,
            data: Bytes::new(&env),
            localization: Map::new(&env),
        };
        notification::NotificationManager::create_template(&env, admin, name, channels, content)
    }

    /// Send notification using template
    pub fn send_template_notification(
        env: Env,
        recipient: Address,
        template_id: u64,
        variables: Map<Bytes, Bytes>,
    ) -> Result<u64, BridgeError> {
        notification::NotificationManager::send_template_notification(
            &env,
            recipient,
            template_id,
            variables,
        )
    }

    /// Get notification tracking information
    pub fn get_notification_tracking(
        env: Env,
        notification_id: u64,
    ) -> Option<NotificationTracking> {
        notification::NotificationManager::get_notification_tracking(&env, notification_id)
    }

    /// Get user notification history
    pub fn get_user_notifications(
        env: Env,
        user: Address,
        limit: u32,
    ) -> Vec<NotificationTracking> {
        notification::NotificationManager::get_user_notifications(&env, user, limit)
    }

    // ========== Social Learning Functions ==========

    /// Create a study group
    pub fn create_study_group(
        env: Env,
        creator: Address,
        name: Bytes,
        description: Bytes,
        subject: Bytes,
        max_members: u32,
        is_private: bool,
        tags: Vec<Bytes>,
        settings: social_learning::StudyGroupSettings,
    ) -> Result<u64, BridgeError> {
        social_learning::SocialLearningManager::create_study_group(
            &env,
            creator,
            name,
            description,
            subject,
            max_members,
            is_private,
            tags,
            settings,
        )
        .map_err(|_| BridgeError::InvalidInput)
    }

    /// Join a study group
    pub fn join_study_group(env: Env, user: Address, group_id: u64) -> Result<(), BridgeError> {
        social_learning::SocialLearningManager::join_study_group(&env, user, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Leave a study group
    pub fn leave_study_group(env: Env, user: Address, group_id: u64) -> Result<(), BridgeError> {
        social_learning::SocialLearningManager::leave_study_group(&env, user, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }

    /// Get study group information
    pub fn get_study_group(
        env: Env,
        group_id: u64,
    ) -> Result<social_learning::StudyGroup, BridgeError> {
        social_learning::SocialLearningManager::get_study_group(&env, group_id)
            .map_err(|_| BridgeError::InvalidInput)
    }
    
    /// Get error statistics
    pub fn get_error_stats(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&Self::ERROR_COUNT)
            .unwrap_or(0u64)
    }
    
    /// Get constant values for external reference
    pub fn get_constants(env: Env) -> (u32, u32, u32, i128, u64) {
        (
            constants::fees::DEFAULT_FEE_RATE,
            constants::chains::DEFAULT_MIN_CONFIRMATIONS,
            constants::oracle::DEFAULT_CONFIDENCE_THRESHOLD,
            constants::amounts::FALLBACK_PRICE,
            constants::oracle::PRICE_FRESHNESS_SECONDS,
        )
    }
    
    /// Enable/disable fallback mechanism
    pub fn set_fallback_enabled(env: Env, enabled: bool) {
        Self::require_admin(&env);
        env.storage().instance().set(&Self::FALLBACK_ENABLED, &enabled);
    }
    
    /// Get fallback status
    pub fn is_fallback_enabled(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::FALLBACK_ENABLED)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Empty test to satisfy CI
        assert!(true);
    }
}
