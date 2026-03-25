#![allow(clippy::all)]
#![allow(unused)]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Map, String, Symbol, Vec};

mod analytics;
mod arbitration;
mod atomic_swap;
mod audit;
mod bft_consensus;
mod bridge;
mod emergency;
mod errors;
mod escrow;
mod escrow_analytics;
mod events;
mod liquidity;
mod message_passing;
mod multichain;
mod notification;
mod notification_events_basic;
mod notification_types;
mod reporting;
mod rewards;
mod slashing;
mod storage;
mod tokenization;
mod types;
pub mod validation;

pub use crate::types::{
    ColorBlindMode, ComponentConfig, DeviceInfo, FeedbackCategory, FocusStyle, FontSize,
    LayoutDensity, MobileAccessibilitySettings, MobilePreferences, NetworkType,
    OnboardingStage, OnboardingStatus, ThemePreference, UserFeedback, VideoQuality,
    BridgeTransaction, ChainConfig, ContentMetadata, ContentToken, CrossChainMessage,
    CrossChainPacket, LiquidityPool, LPPosition, MessageReceipt, MobileAnalytics,
    MobileCommunity, MobilePaymentMethod, MobileSecuritySettings, MultiChainAsset,
    NotificationChannel, NotificationPreference, PacketStatus, PushNotification,
    ValidatorSignature, VisualizationDataPoint,
};
pub use errors::{BridgeError, EscrowError, MobilePlatformError, RewardsError};
pub use types::UserReputation;

/// TeachLink main contract.
///
/// This contract provides entry points for all TeachLink functionality
/// including bridging, rewards, escrow, tokenization, and reputation.
#[contract]
pub struct TeachLinkBridge;

#[contractimpl]
impl TeachLinkBridge {
    /// Initialize the bridge contract
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        min_validators: u32,
        fee_recipient: Address,
    ) -> Result<(), BridgeError> {
        bridge::Bridge::initialize(&env, token, admin, min_validators, fee_recipient)
    }

    /// Bridge tokens out to another chain
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> Result<u64, BridgeError> {
        bridge::Bridge::bridge_out(&env, from, amount, destination_chain, destination_address)
    }

    /// Complete a bridge transaction
    pub fn complete_bridge(
        env: Env,
        message: types::CrossChainMessage,
        validator_signatures: Vec<Address>,
    ) -> Result<(), BridgeError> {
        bridge::Bridge::complete_bridge(&env, message, validator_signatures)
    }

    /// Get bridge transaction
    pub fn get_bridge_transaction(env: Env, nonce: u64) -> Option<BridgeTransaction> {
        bridge::Bridge::get_bridge_transaction(&env, nonce)
    }

    /// Get nonce
    pub fn get_nonce(env: Env) -> u64 {
        bridge::Bridge::get_nonce(&env)
    }

    /// Get token address
    pub fn get_token(env: Env) -> Address {
        bridge::Bridge::get_token(&env)
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        bridge::Bridge::get_admin(&env)
    }
}
