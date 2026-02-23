use soroban_sdk::{contracttype, Address, String, Vec};
use crate::types::*;

// ========== CDN Management Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CDNInitializedEvent {
    pub admin: Address,
    pub primary_region: String,
    pub max_nodes: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeRegisteredEvent {
    pub node_id: String,
    pub region: String,
    pub node_type: CDNNodeType,
    pub capacity: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeDeactivatedEvent {
    pub node_id: String,
    pub reason: String,
    pub timestamp: u64,
}

// ========== Content Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentUploadedEvent {
    pub content_id: String,
    pub uploader: Address,
    pub content_type: ContentType,
    pub size: u64,
    pub replicas: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentAccessedEvent {
    pub content_id: String,
    pub node_id: String,
    pub user_location: String,
    pub bytes_served: u64,
    pub response_time: u64,
    pub cache_status: CacheStatus,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentReplicatedEvent {
    pub content_id: String,
    pub source_node: String,
    pub target_node: String,
    pub timestamp: u64,
}

// ========== Optimization Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationAppliedEvent {
    pub content_id: String,
    pub optimization_type: OptimizationType,
    pub old_size: u64,
    pub new_size: u64,
    pub savings_percentage: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachePolicyUpdatedEvent {
    pub content_id: String,
    pub old_policy: CachePolicy,
    pub new_policy: CachePolicy,
    pub timestamp: u64,
}

// ========== Security Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DRMEnabledEvent {
    pub content_id: String,
    pub admin: Address,
    pub encryption_enabled: bool,
    pub watermarking_enabled: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessTokenGeneratedEvent {
    pub token_id: String,
    pub content_id: String,
    pub user: Address,
    pub expires_at: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityViolationEvent {
    pub content_id: String,
    pub violation_type: String,
    pub user_location: String,
    pub blocked: bool,
    pub timestamp: u64,
}

// ========== Disaster Recovery Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupCreatedEvent {
    pub backup_id: String,
    pub content_id: String,
    pub backup_regions: Vec<String>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupRestoredEvent {
    pub backup_id: String,
    pub content_id: String,
    pub target_region: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailoverTriggeredEvent {
    pub failed_node_id: String,
    pub backup_node_id: String,
    pub affected_content: Vec<String>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPlanExecutedEvent {
    pub plan_id: String,
    pub failed_region: String,
    pub recovery_time: u64,
    pub timestamp: u64,
}

// ========== Analytics Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricsUpdatedEvent {
    pub metric_type: String,
    pub region: Option<String>,
    pub value: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceAlertEvent {
    pub alert_type: String,
    pub node_id: String,
    pub metric_value: u64,
    pub threshold: u64,
    pub timestamp: u64,
}
// ========== Enhanced Streaming Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingConfigCreatedEvent {
    pub content_id: String,
    pub protocol: StreamingProtocol,
    pub profile_count: u32,
    pub segment_duration: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAdaptedEvent {
    pub content_id: String,
    pub user: Address,
    pub old_quality: StreamingQuality,
    pub new_quality: StreamingQuality,
    pub network_bandwidth: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingManifestGeneratedEvent {
    pub content_id: String,
    pub protocol: StreamingProtocol,
    pub profile_count: u32,
    pub network_bandwidth: u64,
    pub timestamp: u64,
}

// ========== Enhanced Cost Optimization Events ==========

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetAlertEvent {
    pub alert_type: String,
    pub current_spend: u64,
    pub budget_limit: u64,
    pub spend_percentage: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostOptimizationAppliedEvent {
    pub optimization_type: OptimizationType,
    pub content_count: u32,
    pub estimated_savings: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricingModelUpdatedEvent {
    pub bandwidth_cost_per_gb: u64,
    pub storage_cost_per_gb: u64,
    pub request_cost_per_1000: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostMetricsCalculatedEvent {
    pub total_cost: u64,
    pub bandwidth_cost: u64,
    pub storage_cost: u64,
    pub request_cost: u64,
    pub efficiency_score: u32,
    pub timestamp: u64,
}