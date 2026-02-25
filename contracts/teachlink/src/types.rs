//! TeachLink Contract Types
//!
//! This module defines all data structures used throughout the TeachLink smart contract.

use soroban_sdk::{contracttype, panic_with_error, Address, Bytes, Map, String, Symbol, Vec};

// Include notification types
pub use crate::notification_types::*;

// ========== Chain Configuration Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u32,
    pub chain_name: Bytes,
    pub is_active: bool,
    pub bridge_contract_address: Bytes,
    pub confirmation_blocks: u32,
    pub gas_price: u64,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiChainAsset {
    pub asset_id: Bytes,
    pub stellar_token: Address,
    pub chain_configs: Map<u32, ChainAssetInfo>,
    pub total_bridged: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainAssetInfo {
    pub chain_id: u32,
    pub token_address: Bytes,
    pub decimals: u32,
    pub is_active: bool,
}

// ========== BFT Consensus Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake: i128,
    pub reputation_score: u32,
    pub is_active: bool,
    pub joined_at: u64,
    pub last_activity: u64,
    pub total_validations: u64,
    pub missed_validations: u64,
    pub slashed_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeProposal {
    pub proposal_id: u64,
    pub message: CrossChainMessage,
    pub votes: Map<Address, bool>,
    pub vote_count: u32,
    pub required_votes: u32,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsensusState {
    pub total_stake: i128,
    pub active_validators: u32,
    pub byzantine_threshold: u32,
    pub last_consensus_round: u64,
}

// ========== Slashing and Rewards Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashingRecord {
    pub validator: Address,
    pub amount: i128,
    pub reason: SlashingReason,
    pub timestamp: u64,
    pub evidence: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlashingReason {
    DoubleVote,
    InvalidSignature,
    Inactivity,
    ByzantineBehavior,
    MaliciousProposal,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorReward {
    pub validator: Address,
    pub amount: i128,
    pub reward_type: RewardType,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RewardType {
    Validation,
    Consensus,
    Uptime,
    Security,
}

// ========== Liquidity and AMM Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityPool {
    pub chain_id: u32,
    pub token: Address,
    pub total_liquidity: i128,
    pub available_liquidity: i128,
    pub locked_liquidity: i128,
    pub lp_providers: Map<Address, LPPosition>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LPPosition {
    pub provider: Address,
    pub amount: i128,
    pub share_percentage: u32,
    pub deposited_at: u64,
    pub rewards_earned: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeFeeStructure {
    pub base_fee: i128,
    pub dynamic_multiplier: u32,
    pub congestion_multiplier: u32,
    pub volume_discount_tiers: Map<u32, u32>,
    pub last_updated: u64,
}

// ========== Message Passing Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainPacket {
    pub packet_id: u64,
    pub source_chain: u32,
    pub destination_chain: u32,
    pub sender: Bytes,
    pub recipient: Bytes,
    pub payload: Bytes,
    pub nonce: u64,
    pub timeout: u64,
    pub status: PacketStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PacketStatus {
    Pending,
    Delivered,
    Failed,
    TimedOut,
    Retrying,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReceipt {
    pub packet_id: u64,
    pub delivered_at: u64,
    pub gas_used: u64,
    pub result: Bytes,
}

// ========== Emergency and Security Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyState {
    pub is_paused: bool,
    pub paused_at: u64,
    pub paused_by: Address,
    pub reason: Bytes,
    pub affected_chains: Vec<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreaker {
    pub chain_id: u32,
    pub max_daily_volume: i128,
    pub current_daily_volume: i128,
    pub max_transaction_amount: i128,
    pub last_reset: u64,
    pub is_triggered: bool,
}

// ========== Audit and Compliance Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRecord {
    pub record_id: u64,
    pub operation_type: OperationType,
    pub operator: Address,
    pub timestamp: u64,
    pub details: Bytes,
    pub tx_hash: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    BridgeIn,
    BridgeOut,
    ValidatorAdded,
    ValidatorRemoved,
    ValidatorSlashed,
    EmergencyPause,
    EmergencyResume,
    FeeUpdate,
    ConfigUpdate,
    BackupCreated,
    BackupVerified,
    RecoveryExecuted,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceReport {
    pub report_id: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub total_volume: i128,
    pub total_transactions: u64,
    pub unique_users: u32,
    pub validator_performance: Map<Address, u32>,
}

// ========== Atomic Swap Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicSwap {
    pub swap_id: u64,
    pub initiator: Address,
    pub initiator_token: Address,
    pub initiator_amount: i128,
    pub counterparty: Address,
    pub counterparty_token: Address,
    pub counterparty_amount: i128,
    pub hashlock: Bytes,
    pub timelock: u64,
    pub status: SwapStatus,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwapStatus {
    Initiated,
    CounterpartyAccepted,
    Completed,
    Refunded,
    Expired,
}

// ========== Analytics Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMetrics {
    pub total_volume: i128,
    pub total_transactions: u64,
    pub active_validators: u32,
    pub average_confirmation_time: u64,
    pub success_rate: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainMetrics {
    pub chain_id: u32,
    pub volume_in: i128,
    pub volume_out: i128,
    pub transaction_count: u64,
    pub average_fee: i128,
    pub last_updated: u64,
}

/// Cached bridge summary for performance: health score and top chains by volume.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedBridgeSummary {
    pub health_score: u32,
    pub top_chains: Vec<(u32, i128)>,
    pub computed_at: u64,
}

// ========== Validator Signature Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Bytes,
    pub timestamp: u64,
}

// ========== Content Tokenization Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentTokenParameters {
    pub creator: Address,
    pub title: Bytes,
    pub description: Bytes,
    pub content_type: ContentType,
    pub content_hash: Bytes,
    pub license_type: Bytes,
    pub tags: Vec<Bytes>,
    pub is_transferable: bool,
    pub royalty_percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowRole {
    Primary,    // Full control, high weight
    Secondary,  // Partial control, low weight
    Arbitrator, // Can resolve disputes
    Auditor,    // Read-only, can only flag issues
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowSigner {
    pub address: Address,
    pub role: EscrowRole,
    pub weight: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowParameters {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub signers: Vec<EscrowSigner>,
    pub threshold: u32,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
}

// ========== Bridge Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeTransaction {
    pub nonce: u64,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub destination_chain: u32,
    pub destination_address: Bytes,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainMessage {
    pub source_chain: u32,
    pub source_tx_hash: Bytes,
    pub nonce: u64,
    pub token: Address,
    pub amount: i128,
    pub recipient: Address,
    pub destination_chain: u32,
}

//
// ==========================
// Rewards Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReward {
    pub user: Address,
    pub total_earned: i128,
    pub claimed: i128,
    pub pending: i128,
    pub last_claim_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardRate {
    pub reward_type: String,
    pub rate: i128,
    pub enabled: bool,
}

// ========== Escrow Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub signers: Vec<EscrowSigner>,
    pub threshold: u32,
    pub approval_count: u32,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub dispute_reason: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorProfile {
    pub address: Address,
    pub name: String,
    pub specialization: Vec<String>,
    pub reputation_score: u32,
    pub total_resolved: u64,
    pub dispute_types_handled: Vec<String>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePool {
    pub token: Address,
    pub balance: i128,
    pub premium_rate: u32, // In basis points
    pub total_claims_paid: i128,
    pub max_payout_percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowApprovalKey {
    pub escrow_id: u64,
    pub signer: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowMetrics {
    pub total_escrows: u64,
    pub total_volume: i128,
    pub total_disputes: u64,
    pub total_resolved: u64,
    pub average_resolution_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeOutcome {
    ReleaseToBeneficiary,
    RefundToDepositor,
}

//
// ==========================
// Credit Score / Contribution Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionType {
    Content,
    Code,
    Community,
    Governance,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub contributor: Address,
    pub c_type: ContributionType,
    pub description: Bytes,
    pub timestamp: u64,
    pub points: u64,
}

//
// ==========================
// Reputation Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserReputation {
    pub participation_score: u32,
    pub completion_rate: u32,
    pub contribution_quality: u32,
    pub total_courses_started: u32,
    pub total_courses_completed: u32,
    pub total_contributions: u32,
    pub last_update: u64,
}

//
// ==========================
// Content Tokenization Types
// ==========================
//

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    Course,
    Material,
    Lesson,
    Assessment,
    Certificate,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentMetadata {
    pub title: Bytes,
    pub description: Bytes,
    pub content_type: ContentType,
    pub creator: Address,
    pub content_hash: Bytes,
    pub license_type: Bytes,
    pub tags: Vec<Bytes>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentToken {
    pub token_id: u64,
    pub metadata: ContentMetadata,
    pub owner: Address,
    pub minted_at: u64,
    pub is_transferable: bool,
    pub royalty_percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRecord {
    pub token_id: u64,
    pub from: Option<Address>,
    pub to: Address,
    pub timestamp: u64,
    pub transaction_hash: Bytes,
    pub transfer_type: TransferType,
    pub notes: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    Mint,
    Transfer,
    License,
    Revoke,
}

// ========== Advanced Analytics & Reporting Types ==========

/// Report type for templates and dashboards
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportType {
    BridgeHealth,
    EscrowSummary,
    ComplianceAudit,
    RewardsSummary,
    TokenizationSummary,
    Custom,
}

/// Template for customizable reports (name, type, optional config)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportTemplate {
    pub template_id: u64,
    pub name: Bytes,
    pub report_type: ReportType,
    pub created_by: Address,
    pub created_at: u64,
    pub config: Bytes,
}

/// Scheduled report (template + next run + interval)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportSchedule {
    pub schedule_id: u64,
    pub template_id: u64,
    pub owner: Address,
    pub next_run_at: u64,
    pub interval_seconds: u64,
    pub enabled: bool,
    pub created_at: u64,
}

/// Snapshot of a generated report (for audit trail and export)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportSnapshot {
    pub report_id: u64,
    pub template_id: u64,
    pub report_type: ReportType,
    pub generated_at: u64,
    pub period_start: u64,
    pub period_end: u64,
    pub generated_by: Address,
    /// Serialized summary metrics for visualization (e.g. key-value pairs)
    pub summary: Bytes,
}

/// Report view/usage record for analytics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportUsage {
    pub report_id: u64,
    pub viewer: Address,
    pub viewed_at: u64,
}

/// Comment on a report for collaboration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportComment {
    pub comment_id: u64,
    pub report_id: u64,
    pub author: Address,
    pub body: Bytes,
    pub created_at: u64,
}

/// Alert rule for real-time reporting (condition type + threshold)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertConditionType {
    BridgeHealthBelow,
    EscrowDisputeRateAbove,
    VolumeAbove,
    VolumeBelow,
    TransactionCountAbove,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlertRule {
    pub rule_id: u64,
    pub name: Bytes,
    pub condition_type: AlertConditionType,
    pub threshold: i128,
    pub owner: Address,
    pub enabled: bool,
    pub created_at: u64,
}

/// Aggregated metrics for dashboard visualization (one series = label + value)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualizationDataPoint {
    pub label: Bytes,
    pub value: i128,
}

/// Dashboard-ready aggregate analytics (bridge, escrow, compliance summary)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardAnalytics {
    pub bridge_health_score: u32,
    pub bridge_total_volume: i128,
    pub bridge_total_transactions: u64,
    pub bridge_success_rate: u32,
    pub escrow_total_count: u64,
    pub escrow_total_volume: i128,
    pub escrow_dispute_count: u64,
    pub escrow_avg_resolution_time: u64,
    pub compliance_report_count: u32,
    pub audit_record_count: u64,
    pub generated_at: u64,
}

// ========== Backup and Disaster Recovery Types ==========

/// RTO tier for recovery time objective (seconds)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RtoTier {
    Critical, // e.g. 300 (5 min)
    High,     // e.g. 3600 (1 hr)
    Standard, // e.g. 86400 (24 hr)
}

/// Backup manifest (metadata for integrity and audit)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupManifest {
    pub backup_id: u64,
    pub created_at: u64,
    pub created_by: Address,
    /// Integrity hash (e.g. hash of critical state snapshot)
    pub integrity_hash: Bytes,
    pub rto_tier: RtoTier,
    /// Encryption/access: 0 = none, non-zero = key version or access policy id
    pub encryption_ref: u64,
}

/// Scheduled backup config
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupSchedule {
    pub schedule_id: u64,
    pub owner: Address,
    pub next_run_at: u64,
    pub interval_seconds: u64,
    pub rto_tier: RtoTier,
    pub enabled: bool,
    pub created_at: u64,
}

/// Recovery record for audit trail and RTO tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryRecord {
    pub recovery_id: u64,
    pub backup_id: u64,
    pub executed_at: u64,
    pub executed_by: Address,
    /// Recovery duration in seconds (RTO measurement)
    pub recovery_duration_secs: u64,
    pub success: bool,
}

// ========== Mobile Platform Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutDensity {
    Compact,
    Standard,
    Comfortable,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FocusStyle {
    Default,
    HighVisibility,
    Solid,
    Dotted,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorBlindMode {
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
    Grayscale,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OnboardingStage {
    ProfileSetup,
    WalletConnect,
    FirstCourse,
    CommunityJoin,
    SecuritySetup,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedbackCategory {
    UX,
    Performance,
    Content,
    Bug,
    FeatureRequest,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecurityEventType {
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    BiometricUsed,
    DeviceAdded,
    DeviceRemoved,
    SuspiciousActivity,
    RemoteWipe,
    PasswordChange,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileLocationData {
    pub latitude: i64,
    pub longitude: i64,
    pub accuracy: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileProfile {
    pub user: Address,
    pub device_info: DeviceInfo,
    pub preferences: MobilePreferences,
    pub offline_settings: OfflineSettings,
    pub notification_preferences: NotificationPreferences,
    pub security_settings: MobileSecuritySettings,
    pub payment_methods: Vec<MobilePaymentMethod>,
    pub accessibility_settings: MobileAccessibilitySettings,
    pub last_sync: u64,
    pub data_usage: DataUsageTracking,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPreferences {
    pub learning_reminders: bool,
    pub deadline_alerts: bool,
    pub achievement_notifications: bool,
    pub social_updates: bool,
    pub content_updates: bool,
    pub quiet_hours: TimeRange,
    pub frequency_limit: u32,
    pub sound_enabled: bool,
    pub vibration_enabled: bool,
    pub led_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    pub start_hour: u32,
    pub end_hour: u32,
    pub timezone: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobilePaymentMethod {
    pub id: u64,
    pub name: Bytes,
    pub method_type: PaymentMethodType,
    pub is_default: bool,
    pub last_used: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentMethodType {
    StellarAsset,
    NativeToken,
    ExternalProvider,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataUsageTracking {
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub cached_data: u64,
    pub streaming_data: u64,
    pub last_reset: u64,
    pub daily_limit: u64,
    pub warning_threshold: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkAnalytics {
    pub connection_type_distribution: Map<NetworkType, u32>,
    pub average_download_speed: u64, // Kbps
    pub average_upload_speed: u64,
    pub connection_stability: u64, // Basis points
    pub offline_duration: u64,     // Minutes per day
    pub roaming_usage: u64,        // Bytes
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkType {
    WiFi,
    Cellular4G,
    Cellular5G,
    Ethernet,
    Offline,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncStrategy {
    WiFiOnly,
    WiFiAndCellular,
    Manual,
    SmartAdaptive,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OfflineQuality {
    TextOnly,
    LowQuality,
    StandardQuality,
    HighQuality,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceInfo {
    pub device_id: Bytes,
    pub model: Bytes,
    pub os_version: Bytes,
    pub push_token: Bytes,
    pub screen_resolution: Bytes,
    pub is_tablet: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobilePreferences {
    pub data_saver_mode: bool,
    pub auto_download_wifi: bool,
    pub video_quality: VideoQuality,
    pub font_size: FontSize,
    pub theme: ThemePreference,
    pub language: Bytes,
    pub vibration_enabled: bool,
    pub sound_enabled: bool,
    pub gesture_navigation: bool,
    pub custom_theme_colors: Map<Symbol, Bytes>, // Color key -> hex
    pub layout_density: LayoutDensity,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VideoQuality {
    Auto,
    Low,
    Medium,
    High,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
    OLED,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileAccessibilitySettings {
    pub screen_reader_enabled: bool,
    pub high_contrast_enabled: bool,
    pub large_text_enabled: bool,
    pub voice_control_enabled: bool,
    pub gesture_navigation_enabled: bool,
    pub haptic_feedback_enabled: bool,
    pub color_blind_mode: ColorBlindMode,
    pub reduced_motion_enabled: bool,
    pub focus_indicator_style: FocusStyle,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileSecuritySettings {
    pub biometric_enabled: bool,
    pub biometric_type: BiometricType,
    pub pin_required: bool,
    pub two_factor_enabled: bool,
    pub session_timeout: u32, // Minutes
    pub encryption_enabled: bool,
    pub remote_wipe_enabled: bool,
    pub trusted_devices: Vec<Bytes>,
    pub login_attempts: u32,
    pub max_login_attempts: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BiometricType {
    Fingerprint,
    FaceID,
    Voice,
    Iris,
    Pattern,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileStatistics {
    pub session_count: u32,
    pub total_time_spent: u64,       // Seconds
    pub average_session_length: u32, // Seconds
    pub active_days_streak: u32,
    pub last_active: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineSettings {
    pub auto_download_enabled: bool,
    pub download_quality: OfflineQuality,
    pub storage_limit: u64,
    pub sync_strategy: SyncStrategy,
    pub offline_duration: u64,
    pub priority_content: Vec<Bytes>,
    pub compression_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingStatus {
    pub user: Address,
    pub completed_stages: Vec<OnboardingStage>,
    pub current_stage: OnboardingStage,
    pub last_updated: u64,
    pub skipped: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UXExperiment {
    pub experiment_id: u64,
    pub name: Symbol,
    pub description: Bytes,
    pub variant_allocations: Map<Address, Symbol>, // User -> Variant Key
    pub start_date: u64,
    pub end_date: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserFeedback {
    pub id: u64,
    pub user: Address,
    pub rating: u32, // 1-5
    pub comment: Bytes,
    pub timestamp: u64,
    pub category: FeedbackCategory,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentConfig {
    pub spacing_unit: u32,
    pub border_radius_base: u32,
    pub transition_duration_base: u32,
    pub elevation_steps: Vec<u32>,
    pub typography_scale: Map<Symbol, u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineContent {
    pub content_id: u64,
    pub content_type: OfflineContentType,
    pub local_path: Bytes,
    pub file_size: u64,
    pub compressed_size: u64,
    pub download_date: u64,
    pub expiry_date: u64,
    pub is_available: bool,
    pub version: u32,
    pub dependencies: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OfflineContentType {
    VideoLesson,
    AudioLesson,
    TextDocument,
    Quiz,
    InteractiveExercise,
    EBook,
    CourseMaterial,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncQueue {
    pub pending_uploads: Vec<SyncItem>,
    pub pending_downloads: Vec<SyncItem>,
    pub conflict_resolution: Vec<SyncConflict>,
    pub last_sync_attempt: u64,
    pub sync_status: SyncStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncItem {
    pub id: u64,
    pub item_type: SyncItemType,
    pub local_path: Bytes,
    pub remote_path: Bytes,
    pub priority: SyncPriority,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncItemType {
    ProgressData,
    QuizResults,
    Notes,
    Bookmarks,
    Certificates,
    UserPreferences,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncConflict {
    pub conflict_id: u64,
    pub item_type: SyncItemType,
    pub local_version: Bytes,
    pub remote_version: Bytes,
    pub conflict_reason: Bytes,
    pub resolution_strategy: ConflictResolution,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictResolution {
    LocalWins,
    RemoteWins,
    Merge,
    ManualReview,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncStatus {
    Idle,
    InProgress,
    Completed,
    Failed,
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushNotification {
    pub id: u64,
    pub user: Address,
    pub notification_type: NotificationType,
    pub title: Bytes,
    pub message: Bytes,
    pub data: Map<Bytes, Bytes>, // Additional data
    pub priority: NotificationPriority,
    pub scheduled_time: u64,
    pub expiry_time: u64,
    pub is_read: bool,
    pub action_buttons: Vec<NotificationAction>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationType {
    LearningReminder,
    DeadlineAlert,
    AchievementUnlocked,
    SocialUpdate,
    ContentUpdate,
    SystemMessage,
    PaymentRequired,
    CourseUpdate,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationAction {
    pub action_id: Bytes,
    pub label: Bytes,
    pub url: Option<Bytes>,
    pub auto_dismiss: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileTransaction {
    pub id: u64,
    pub user: Address,
    pub payment_method_id: u64,
    pub amount: u64,
    pub currency: Bytes,
    pub description: Bytes,
    pub merchant: Bytes,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub confirmation_code: Bytes,
    pub fraud_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Refunded,
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityEvent {
    pub id: u64,
    pub user: Address,
    pub event_type: SecurityEventType,
    pub device_id: Bytes,
    pub has_location: bool,
    pub location_lat: i64,
    pub location_lon: i64,
    pub location_accuracy: u64,
    pub location_ts: u64,
    pub timestamp: u64,
    pub severity: SecuritySeverity,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileAnalytics {
    pub user: Address,
    pub device_analytics: DeviceAnalytics,
    pub usage_analytics: UsageAnalytics,
    pub performance_analytics: PerformanceAnalytics,
    pub engagement_analytics: EngagementAnalytics,
    pub network_analytics: NetworkAnalytics,
    pub error_tracking: ErrorTracking,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceAnalytics {
    pub app_version: Bytes,
    pub os_version: Bytes,
    pub device_model: Bytes,
    pub screen_resolution: Bytes,
    pub memory_usage: u64,
    pub storage_usage: u64,
    pub battery_level: u32,
    pub thermal_state: ThermalState,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThermalState {
    Normal,
    Warm,
    Hot,
    Critical,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageAnalytics {
    pub session_duration: u32, // Average minutes
    pub sessions_per_day: u32,
    pub active_days_per_week: u32,
    pub peak_usage_hours: Vec<u32>,
    pub feature_usage: Map<Bytes, u32>,
    pub screen_time: u64,      // Total minutes
    pub data_consumption: u64, // Bytes
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceAnalytics {
    pub app_load_time: u32, // Milliseconds
    pub screen_render_time: u32,
    pub network_latency: u32,
    pub crash_count: u32,
    pub anr_count: u32, // Application Not Responding
    pub memory_leaks: u32,
    pub battery_drain_rate: u64, // Basis points per hour
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngagementAnalytics {
    pub lesson_completion_rate: u64, // Basis points
    pub quiz_attempt_rate: u64,      // Basis points
    pub social_interaction_count: u32,
    pub feedback_submission_rate: u64,          // Basis points
    pub push_notif_response_rate: u64,          // Basis points
    pub feature_adoption_rate: Map<Bytes, u64>, // Basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorTracking {
    pub crash_reports: Vec<CrashReport>,
    pub anr_reports: Vec<ANRReport>,
    pub network_errors: Vec<NetworkError>,
    pub user_reported_issues: Vec<UserIssue>,
    pub error_rate: u64, // Basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrashReport {
    pub id: u64,
    pub timestamp: u64,
    pub app_version: Bytes,
    pub device_info: Bytes,
    pub stack_trace: Bytes,
    pub user_action: Bytes,
    pub reproducible: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ANRReport {
    pub id: u64,
    pub timestamp: u64,
    pub duration: u32, // Seconds
    pub app_state: Bytes,
    pub device_load: u64, // Basis points
    pub user_action: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkError {
    pub id: u64,
    pub timestamp: u64,
    pub error_type: NetworkErrorType,
    pub url: Bytes,
    pub response_code: u32,
    pub retry_count: u32,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkErrorType {
    Timeout,
    ConnectionRefused,
    DNSFailure,
    SSLHandshakeFailed,
    ServerError,
    ClientError,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserIssue {
    pub id: u64,
    pub timestamp: u64,
    pub issue_type: Bytes,
    pub description: Bytes,
    pub severity: u32,
    pub user_email: Option<Bytes>,
    pub resolved: bool,
    pub resolution: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileCommunity {
    pub user: Address,
    pub mobile_groups: Vec<MobileGroup>,
    pub location_sharing: LocationSharingSettings,
    pub quick_actions: Vec<QuickAction>,
    pub mobile_challenges: Vec<MobileChallenge>,
    pub social_features: MobileSocialFeatures,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileGroup {
    pub id: u64,
    pub name: Bytes,
    pub description: Bytes,
    pub members: Vec<Address>,
    pub is_location_based: bool,
    pub meeting_locations: Vec<MobileLocationData>,
    pub mobile_specific_features: Vec<MobileFeature>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileFeature {
    LocationCheckIn,
    VoiceNotes,
    PhotoSharing,
    QuickPolls,
    EmergencyAlerts,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocationSharingSettings {
    pub enabled: bool,
    pub sharing_duration: u64, // Hours
    pub trusted_contacts: Vec<Address>,
    pub accuracy_level: LocationAccuracy,
    pub auto_check_in: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocationAccuracy {
    Exact,
    Approximate,
    CityLevel,
    Disabled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuickAction {
    pub id: u64,
    pub name: Bytes,
    pub icon: Bytes,
    pub action_type: QuickActionType,
    pub target_screen: Bytes,
    pub parameters: Map<Bytes, Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuickActionType {
    StartLesson,
    JoinStudyGroup,
    TakeQuiz,
    ViewProgress,
    ContactMentor,
    ScheduleReminder,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileChallenge {
    pub id: u64,
    pub title: Bytes,
    pub description: Bytes,
    pub challenge_type: ChallengeType,
    pub requirements: Vec<Bytes>,
    pub rewards: ChallengeReward,
    pub participants: Vec<Address>,
    pub start_date: u64,
    pub end_date: u64,
    pub is_mobile_specific: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChallengeType {
    DailyStreak,
    WeeklyGoal,
    SocialLearning,
    LocationBased,
    SkillMastery,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChallengeReward {
    pub reward_type: MobileRewardType,
    pub amount: u64,
    pub badge: Option<Bytes>,
    pub certificate: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileRewardType {
    Points,
    Badge,
    Certificate,
    Discount,
    PremiumAccess,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileSocialFeatures {
    pub voice_notes_enabled: bool,
    pub photo_sharing_enabled: bool,
    pub location_checkins_enabled: bool,
    pub quick_polls_enabled: bool,
    pub emergency_contacts: Vec<Address>,
    pub study_buddies: Vec<Address>,
    pub mentor_quick_connect: bool,
}
