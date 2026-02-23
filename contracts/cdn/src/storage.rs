use soroban_sdk::{symbol_short, Symbol};

// ========== Configuration Storage Keys ==========
pub const CDN_CONFIG: Symbol = symbol_short!("CDN_CFG");
pub const CDN_ADMIN: Symbol = symbol_short!("CDN_ADM");

// ========== Node Management Storage Keys ==========
pub const CDN_NODES: Symbol = symbol_short!("NODES");
pub const NODE_COUNT: Symbol = symbol_short!("NODE_CNT");
pub const ACTIVE_NODES: Symbol = symbol_short!("ACT_NODES");
pub const REGIONAL_NODES: Symbol = symbol_short!("REG_NODES");

// ========== Content Storage Keys ==========
pub const CONTENT_ITEMS: Symbol = symbol_short!("CONTENT");
pub const CONTENT_COUNT: Symbol = symbol_short!("CNT_CNT");
pub const CONTENT_REPLICAS: Symbol = symbol_short!("REPLICAS");
pub const CONTENT_METADATA: Symbol = symbol_short!("METADATA");

// ========== Analytics Storage Keys ==========
pub const GLOBAL_METRICS: Symbol = symbol_short!("GLB_MET");
pub const CONTENT_ANALYTICS: Symbol = symbol_short!("CNT_ANA");
pub const REGIONAL_METRICS: Symbol = symbol_short!("REG_MET");
pub const ACCESS_LOGS: Symbol = symbol_short!("ACC_LOGS");
pub const BANDWIDTH_USAGE: Symbol = symbol_short!("BW_USAGE");

// ========== Cache Storage Keys ==========
pub const CACHE_POLICIES: Symbol = symbol_short!("CACHE_POL");
pub const CACHE_STATUS: Symbol = symbol_short!("CACHE_ST");
pub const CACHE_METRICS: Symbol = symbol_short!("CACHE_MET");

// ========== Optimization Storage Keys ==========
pub const OPTIMIZATION_CONFIGS: Symbol = symbol_short!("OPT_CFG");
pub const COMPRESSION_SETTINGS: Symbol = symbol_short!("COMP_SET");
pub const COST_METRICS: Symbol = symbol_short!("COST_MET");

// ========== Security Storage Keys ==========
pub const DRM_CONFIGS: Symbol = symbol_short!("DRM_CFG");
pub const ACCESS_TOKENS: Symbol = symbol_short!("ACC_TOK");
pub const ENCRYPTION_KEYS: Symbol = symbol_short!("ENC_KEYS");
pub const SECURITY_POLICIES: Symbol = symbol_short!("SEC_POL");

// ========== Disaster Recovery Storage Keys ==========
pub const BACKUP_RECORDS: Symbol = symbol_short!("BACKUPS");
pub const RECOVERY_PLANS: Symbol = symbol_short!("REC_PLAN");
pub const BACKUP_COUNTER: Symbol = symbol_short!("BKP_CNT");
pub const RECOVERY_LOGS: Symbol = symbol_short!("REC_LOGS");

// ========== Event Storage Keys ==========
pub const EVENT_COUNTER: Symbol = symbol_short!("EVT_CNT");
pub const RECENT_EVENTS: Symbol = symbol_short!("REC_EVT");

// ========== Streaming Storage Keys ==========
pub const STREAMING_CONFIGS: Symbol = symbol_short!("STR_CFG");
pub const ADAPTIVE_PROFILES: Symbol = symbol_short!("ADP_PRF");
pub const MANIFEST_CACHE: Symbol = symbol_short!("MAN_CACHE");
// ========== Enhanced Streaming Storage Keys ==========
pub const NETWORK_CONDITIONS: Symbol = symbol_short!("NET_COND");
pub const STREAMING_SESSIONS: Symbol = symbol_short!("STR_SESS");
pub const QUALITY_ADAPTATIONS: Symbol = symbol_short!("QUAL_ADP");

// ========== Enhanced Cost Optimization Storage Keys ==========
pub const PRICING_MODELS: Symbol = symbol_short!("PRICING");
pub const COST_BUDGETS: Symbol = symbol_short!("BUDGETS");
pub const COST_ALERTS: Symbol = symbol_short!("CST_ALRT");
pub const COST_HISTORY: Symbol = symbol_short!("CST_HIST");
