use soroban_sdk::{contracttype, Address, Bytes, Map, String, Vec};

// ========== Core CDN Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CDNConfig {
    pub admin: Address,
    pub primary_region: String,
    pub max_nodes: u32,
    pub initialized: bool,
    pub total_nodes: u32,
    pub total_content: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CDNNodeType {
    Edge,      // Edge cache servers for fast delivery
    Origin,    // Origin servers storing original content
    Shield,    // Shield servers for additional protection
    Streaming, // Specialized streaming servers
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CDNNode {
    pub node_id: String,
    pub region: String,
    pub endpoint: String,
    pub node_type: CDNNodeType,
    pub capacity: u64,
    pub current_load: u64,
    pub health_score: u32,
    pub last_health_check: u64,
    pub is_active: bool,
    pub bandwidth_limit: u64,
    pub storage_used: u64,
}

// ========== Content Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    Video,
    Audio,
    Image,
    Document,
    Interactive,
    Archive,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamingQuality {
    Low,      // 480p or equivalent
    Medium,   // 720p or equivalent
    High,     // 1080p or equivalent
    Ultra,    // 4K or equivalent
    Adaptive, // Adaptive bitrate streaming
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamingProtocol {
    HLS,         // HTTP Live Streaming
    DASH,        // Dynamic Adaptive Streaming over HTTP
    WebRTC,      // Real-time streaming
    Progressive, // Progressive download
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingProfile {
    pub quality: StreamingQuality,
    pub bitrate: u32, // in kbps
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub framerate: u32, // fps
    pub codec: String,  // H264, H265, AV1, etc.
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveStreamingConfig {
    pub protocol: StreamingProtocol,
    pub profiles: Vec<StreamingProfile>,
    pub segment_duration: u32, // in seconds
    pub playlist_type: String, // VOD or LIVE
    pub encryption_enabled: bool,
    pub drm_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkCondition {
    pub bandwidth: u64,          // in bps
    pub latency: u64,            // in ms
    pub packet_loss: u32,        // percentage
    pub connection_type: String, // wifi, cellular, ethernet
    pub stability_score: u32,    // 0-100
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingManifest {
    pub manifest_url: String,
    pub protocol: StreamingProtocol,
    pub profiles: Vec<StreamingProfile>,
    pub segment_urls: Vec<String>,
    pub duration: u64, // total duration in seconds
    pub is_live: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CachePolicy {
    NoCache,    // No caching
    ShortTerm,  // Cache for 1 hour
    MediumTerm, // Cache for 24 hours
    LongTerm,   // Cache for 7 days
    Permanent,  // Cache permanently
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CacheStatus {
    Hit,     // Content served from cache
    Miss,    // Content not in cache
    Stale,   // Cached content is stale
    Warming, // Cache is being warmed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentItem {
    pub content_id: String,
    pub content_hash: Bytes,
    pub content_type: ContentType,
    pub size: u64,
    pub uploader: Address,
    pub upload_timestamp: u64,
    pub metadata: Map<String, String>,
    pub cache_policy: CachePolicy,
    pub compression: CompressionType,
    pub replicas: Vec<String>, // List of node IDs where content is replicated
    pub access_count: u64,
    pub last_accessed: u64,
    pub is_encrypted: bool,
    pub drm_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryEndpoint {
    pub url: String,
    pub node_id: String,
    pub region: String,
    pub estimated_latency: u64,
    pub cache_status: CacheStatus,
    pub has_streaming_manifest: bool,
    pub security_token: Option<String>,
}

// ========== Optimization Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Brotli,
    WebP, // For images
    AVIF, // For images
    H264, // For video
    H265, // For video
    AV1,  // For video
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationRecommendation {
    pub recommendation_type: OptimizationType,
    pub description: String,
    pub estimated_savings: u64, // In bytes or percentage
    pub priority: u32,          // 1-10, higher is more important
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptimizationType {
    Compression,
    Caching,
    Replication,
    Format,
    Routing,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostOptimization {
    pub current_cost: u64,
    pub optimized_cost: u64,
    pub savings: u64,
    pub recommendations: Vec<String>,
}

// ========== Analytics Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentAnalytics {
    pub content_id: String,
    pub total_requests: u64,
    pub total_bytes_served: u64,
    pub average_response_time: u64,
    pub cache_hit_ratio: u32, // Percentage
    pub top_regions: Vec<String>,
    pub bandwidth_usage: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalMetrics {
    pub total_requests: u64,
    pub total_bytes_served: u64,
    pub average_response_time: u64,
    pub cache_hit_ratio: u32,
    pub active_nodes: u32,
    pub total_content_items: u64,
    pub bandwidth_usage: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalMetrics {
    pub region: String,
    pub requests: u64,
    pub bytes_served: u64,
    pub average_response_time: u64,
    pub cache_hit_ratio: u32,
    pub active_nodes: u32,
}

// ========== Security Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DRMConfig {
    pub encryption_key: Bytes,
    pub license_server: String,
    pub allowed_domains: Vec<String>,
    pub max_concurrent_streams: u32,
    pub expiry_time: Option<u64>,
    pub watermarking_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessToken {
    pub token_id: String,
    pub content_id: String,
    pub user: Address,
    pub issued_at: u64,
    pub expires_at: u64,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

// ========== Disaster Recovery Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackupStatus {
    InProgress,
    Completed,
    Failed,
    Verifying,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupRecord {
    pub backup_id: String,
    pub content_id: String,
    pub backup_regions: Vec<String>,
    pub created_at: u64,
    pub status: BackupStatus,
    pub integrity_hash: Bytes,
    pub recovery_priority: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPlan {
    pub plan_id: String,
    pub plan_name: String,
    pub critical_content: Vec<String>,
    pub backup_regions: Vec<String>,
    pub recovery_time_objective: u64,
    pub created_at: u64,
    pub last_tested: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeHealthStatus {
    Healthy,
    Warning,
    Critical,
    Failed,
    Maintenance,
}
// ========== Enhanced Cost Optimization Types ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricingModel {
    pub bandwidth_cost_per_gb: u64, // Cost per GB of bandwidth
    pub storage_cost_per_gb: u64,   // Cost per GB of storage
    pub request_cost_per_1000: u64, // Cost per 1000 requests
    pub region_multiplier: u32,     // Regional cost multiplier (percentage)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostMetrics {
    pub total_bandwidth_cost: u64,
    pub total_storage_cost: u64,
    pub total_request_cost: u64,
    pub total_cost: u64,
    pub cost_per_gb_served: u64,
    pub cost_efficiency_score: u32, // 0-100
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetAlert {
    pub alert_type: String, // warning, critical, exceeded
    pub current_spend: u64,
    pub budget_limit: u64,
    pub projected_monthly_cost: u64,
    pub recommendations: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostBudget {
    pub monthly_limit: u64,
    pub current_spend: u64,
    pub alert_thresholds: Vec<u32>, // Alert at these percentages
    pub auto_optimize: bool,        // Auto-apply cost optimizations
}
