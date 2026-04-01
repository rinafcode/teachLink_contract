#![cfg(test)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unreadable_literal)]

//! Gas usage benchmarks for TeachLink contract operations.
//!
//! Measures instruction budgets (gas) consumed by each contract function
//! to detect performance regressions across PRs.

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, Env, Map, Vec};
use teachlink_contract::{
    ContentTokenParameters, ContentType, FontSize, LayoutDensity, MobilePreferences,
    NotificationChannel, TeachLinkBridge, TeachLinkBridgeClient, ThemePreference, VideoQuality,
};

/// Maximum gas budget allowed per operation (in Soroban cost units).
/// Adjust these thresholds as the contract evolves.
mod gas_thresholds {
    /// Contract initialization
    pub const INITIALIZE: u64 = 500_000;
    /// Adding a validator
    pub const ADD_VALIDATOR: u64 = 200_000;
    /// Adding a supported chain
    pub const ADD_SUPPORTED_CHAIN: u64 = 200_000;
    /// Bridge-out (lock tokens)
    pub const BRIDGE_OUT: u64 = 800_000;
    /// Set bridge fee
    pub const SET_BRIDGE_FEE: u64 = 150_000;
    /// Read-only queries (get_token, get_nonce, etc.)
    pub const READ_QUERY: u64 = 100_000;
    /// Cache read (get_cached_bridge_summary hit)
    pub const CACHE_HIT: u64 = 200_000;
    /// Cache miss + compute
    pub const CACHE_MISS_COMPUTE: u64 = 1_500_000;
    /// Register a validator with stake
    pub const REGISTER_VALIDATOR: u64 = 500_000;
    /// Create an atomic swap
    pub const INITIATE_ATOMIC_SWAP: u64 = 900_000;
    /// Mint content token
    pub const MINT_CONTENT_TOKEN: u64 = 600_000;
    /// Transfer content token
    pub const TRANSFER_CONTENT_TOKEN: u64 = 500_000;
    /// Initialize rewards
    pub const INITIALIZE_REWARDS: u64 = 400_000;
    /// Fund reward pool
    pub const FUND_REWARD_POOL: u64 = 500_000;
    /// Issue reward
    pub const ISSUE_REWARD: u64 = 400_000;
    /// Claim rewards
    pub const CLAIM_REWARDS: u64 = 600_000;
    /// Send cross-chain packet
    pub const SEND_PACKET: u64 = 700_000;
    /// Initialize mobile profile
    pub const INIT_MOBILE_PROFILE: u64 = 400_000;
    /// Send notification
    pub const SEND_NOTIFICATION: u64 = 350_000;
    /// Create audit record
    pub const CREATE_AUDIT_RECORD: u64 = 400_000;
    /// WASM binary size threshold (bytes) - 300 KB
    pub const WASM_SIZE_BYTES: u64 = 307_200;
}

/// Helper to measure gas consumed by a closure using the Soroban budget.
fn measure_gas<F: FnMut()>(env: &Env, mut f: F) -> u64 {
    let budget_before = env.budget().cpu_instruction_cost();
    f();
    let budget_after = env.budget().cpu_instruction_cost();
    budget_after.saturating_sub(budget_before)
}

/// Assert gas is within threshold, printing a warning if exceeded.
fn assert_gas_within(name: &str, gas_used: u64, threshold: u64) {
    println!(
        "  [GAS] {}: {} instructions (threshold: {})",
        name, gas_used, threshold
    );
    assert!(
        gas_used <= threshold,
        "GAS REGRESSION: {} used {} instructions, exceeding threshold of {}",
        name,
        gas_used,
        threshold,
    );
}

fn setup_contract(env: &Env) -> TeachLinkBridgeClient<'_> {
    env.mock_all_auths();
    let contract_id = env.register(TeachLinkBridge, ());
    TeachLinkBridgeClient::new(env, &contract_id)
}

// ==========================
// Bridge Operation Benchmarks
// ==========================

#[test]
fn gas_bench_initialize() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let gas = measure_gas(&env, || {
        client.initialize(&token, &admin, &2, &fee_recipient);
    });

    assert_gas_within("initialize", gas, gas_thresholds::INITIALIZE);
}

#[test]
fn gas_bench_add_validator() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let validator = Address::generate(&env);

    let gas = measure_gas(&env, || {
        client.add_validator(&validator);
    });

    assert_gas_within("add_validator", gas, gas_thresholds::ADD_VALIDATOR);
}

#[test]
fn gas_bench_add_supported_chain() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let gas = measure_gas(&env, || {
        client.add_supported_chain(&1u32);
    });

    assert_gas_within(
        "add_supported_chain",
        gas,
        gas_thresholds::ADD_SUPPORTED_CHAIN,
    );
}

#[test]
fn gas_bench_set_bridge_fee() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let gas = measure_gas(&env, || {
        client.set_bridge_fee(&100i128);
    });

    assert_gas_within("set_bridge_fee", gas, gas_thresholds::SET_BRIDGE_FEE);
}

#[test]
fn gas_bench_read_queries() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let gas = measure_gas(&env, || {
        let _ = client.get_token();
        let _ = client.get_nonce();
        let _ = client.get_bridge_fee();
        let _ = client.get_admin();
    });

    assert_gas_within("read_queries_bundle", gas, gas_thresholds::READ_QUERY);
}

// ==========================
// Performance Cache Benchmarks
// ==========================

#[test]
fn gas_bench_cache_compute() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let gas = measure_gas(&env, || {
        let _ = client.compute_and_cache_bridge_summary();
    });

    assert_gas_within(
        "compute_and_cache_bridge_summary",
        gas,
        gas_thresholds::CACHE_MISS_COMPUTE,
    );
}

#[test]
fn gas_bench_cache_hit() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    // Prime the cache
    let _ = client.compute_and_cache_bridge_summary();

    let gas = measure_gas(&env, || {
        let _ = client.get_cached_bridge_summary();
    });

    assert_gas_within("get_cached_bridge_summary", gas, gas_thresholds::CACHE_HIT);
}

// ==========================
// BFT Consensus Benchmarks
// ==========================

#[test]
fn gas_bench_register_validator() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let validator = Address::generate(&env);

    let gas = measure_gas(&env, || {
        let _ = client.try_register_validator(&validator, &100_000_000i128);
    });

    assert_gas_within(
        "register_validator",
        gas,
        gas_thresholds::REGISTER_VALIDATOR,
    );
}

// ==========================
// Content Tokenization Benchmarks
// ==========================

#[test]
fn gas_bench_mint_content_token() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let creator = Address::generate(&env);
    let params = ContentTokenParameters {
        creator: creator.clone(),
        title: Bytes::from_slice(&env, b"Test Course"),
        description: Bytes::from_slice(&env, b"A test course for gas benchmarking"),
        content_type: ContentType::Course,
        content_hash: Bytes::from_slice(&env, b"hash123"),
        license_type: Bytes::from_slice(&env, b"CC-BY"),
        tags: Vec::new(&env),
        is_transferable: true,
        royalty_percentage: 10,
    };

    let gas = measure_gas(&env, || {
        let _ = client.mint_content_token(&params);
    });

    assert_gas_within(
        "mint_content_token",
        gas,
        gas_thresholds::MINT_CONTENT_TOKEN,
    );
}

#[test]
fn gas_bench_transfer_content_token() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let creator = Address::generate(&env);
    let receiver = Address::generate(&env);
    let params = ContentTokenParameters {
        creator: creator.clone(),
        title: Bytes::from_slice(&env, b"Test Course"),
        description: Bytes::from_slice(&env, b"A test course"),
        content_type: ContentType::Course,
        content_hash: Bytes::from_slice(&env, b"hash123"),
        license_type: Bytes::from_slice(&env, b"CC-BY"),
        tags: Vec::new(&env),
        is_transferable: true,
        royalty_percentage: 10,
    };

    let token_id = client.mint_content_token(&params);

    let gas = measure_gas(&env, || {
        client.transfer_content_token(&creator, &receiver, &token_id, &None);
    });

    assert_gas_within(
        "transfer_content_token",
        gas,
        gas_thresholds::TRANSFER_CONTENT_TOKEN,
    );
}

// ==========================
// Notification Benchmarks
// ==========================

#[test]
fn gas_bench_send_notification() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);
    client.initialize_notifications();

    let recipient = Address::generate(&env);

    let gas = measure_gas(&env, || {
        // Use try_ to handle simulated delivery failure in test env
        let _ = client.try_send_notification(
            &recipient,
            &NotificationChannel::InApp,
            &Bytes::from_slice(&env, b"Test Subject"),
            &Bytes::from_slice(&env, b"Test notification body"),
        );
    });

    assert_gas_within("send_notification", gas, gas_thresholds::SEND_NOTIFICATION);
}

// ==========================
// Audit Benchmarks
// ==========================

#[test]
fn gas_bench_create_audit_record() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let operator = Address::generate(&env);

    let gas = measure_gas(&env, || {
        let _ = client.create_audit_record(
            &teachlink_contract::OperationType::BridgeOut,
            &operator,
            &Bytes::from_slice(&env, b"audit details"),
            &Bytes::from_slice(&env, b"tx_hash_value"),
        );
    });

    assert_gas_within(
        "create_audit_record",
        gas,
        gas_thresholds::CREATE_AUDIT_RECORD,
    );
}

// ==========================
// Mobile Platform Benchmarks
// ==========================

#[test]
fn gas_bench_initialize_mobile_profile() {
    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    let user = Address::generate(&env);
    let device_info = teachlink_contract::DeviceInfo {
        device_id: Bytes::from_slice(&env, b"device-001"),
        model: Bytes::from_slice(&env, b"iPhone 14"),
        os_version: Bytes::from_slice(&env, b"iOS 17"),
        push_token: Bytes::from_slice(&env, b"push_token_abc"),
        screen_resolution: Bytes::from_slice(&env, b"1170x2532"),
        is_tablet: false,
    };
    let preferences = MobilePreferences {
        data_saver_mode: false,
        auto_download_wifi: true,
        video_quality: VideoQuality::Auto,
        font_size: FontSize::Medium,
        theme: ThemePreference::System,
        language: Bytes::from_slice(&env, b"en"),
        vibration_enabled: true,
        sound_enabled: true,
        gesture_navigation: true,
        custom_theme_colors: Map::new(&env),
        layout_density: LayoutDensity::Comfortable,
    };

    let gas = measure_gas(&env, || {
        let _ = client.initialize_mobile_profile(&user, &device_info, &preferences);
    });

    assert_gas_within(
        "initialize_mobile_profile",
        gas,
        gas_thresholds::INIT_MOBILE_PROFILE,
    );
}

// ==========================
// Regression Detection Report
// ==========================

#[test]
fn gas_bench_regression_report() {
    println!("\n========================================");
    println!("  GAS BENCHMARK SUMMARY");
    println!("========================================");

    let env = Env::default();
    let client = setup_contract(&env);
    let token = Address::generate(&env);
    let admin = Address::generate(&env);
    let fee_recipient = Address::generate(&env);
    client.initialize(&token, &admin, &2, &fee_recipient);

    println!("  Operation                 | Gas Used    | Threshold   | Status");
    println!("  --------------------------|-------------|-------------|--------");

    // Run each measurement
    let gas = measure_gas(&env, || {
        let _ = client.get_token();
    });
    let status = if gas <= gas_thresholds::READ_QUERY {
        "PASS"
    } else {
        "FAIL"
    };
    println!(
        "  {:<26}| {:>11} | {:>11} | {}",
        "get_token",
        gas,
        gas_thresholds::READ_QUERY,
        status
    );

    let gas = measure_gas(&env, || {
        let _ = client.get_nonce();
    });
    let status = if gas <= gas_thresholds::READ_QUERY {
        "PASS"
    } else {
        "FAIL"
    };
    println!(
        "  {:<26}| {:>11} | {:>11} | {}",
        "get_nonce",
        gas,
        gas_thresholds::READ_QUERY,
        status
    );

    let gas = measure_gas(&env, || {
        let _ = client.get_bridge_fee();
    });
    let status = if gas <= gas_thresholds::READ_QUERY {
        "PASS"
    } else {
        "FAIL"
    };
    println!(
        "  {:<26}| {:>11} | {:>11} | {}",
        "get_bridge_fee",
        gas,
        gas_thresholds::READ_QUERY,
        status
    );

    println!("========================================\n");
}
