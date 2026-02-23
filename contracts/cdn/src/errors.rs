use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CDNError {
    // ========== Initialization Errors ==========
    AlreadyInitialized = 1,
    NotInitialized = 2,
    
    // ========== Authorization Errors ==========
    Unauthorized = 10,
    InvalidAdmin = 11,
    
    // ========== Node Management Errors ==========
    NodeNotFound = 20,
    NodeAlreadyExists = 21,
    NodeCapacityExceeded = 22,
    NodeInactive = 23,
    MaxNodesReached = 24,
    InvalidNodeType = 25,
    
    // ========== Content Management Errors ==========
    ContentNotFound = 30,
    ContentAlreadyExists = 31,
    ContentTooLarge = 32,
    InvalidContentType = 33,
    InvalidContentHash = 34,
    ContentCorrupted = 35,
    
    // ========== Delivery Errors ==========
    NoAvailableNodes = 40,
    DeliveryFailed = 41,
    InvalidEndpoint = 42,
    NetworkError = 43,
    
    // ========== Cache Errors ==========
    CacheError = 50,
    CacheMiss = 51,
    CacheCorrupted = 52,
    InvalidCachePolicy = 53,
    
    // ========== Optimization Errors ==========
    OptimizationFailed = 60,
    UnsupportedCompression = 61,
    CompressionFailed = 62,
    InvalidOptimization = 63,
    
    // ========== Analytics Errors ==========
    AnalyticsError = 70,
    InvalidTimeRange = 71,
    InsufficientData = 72,
    
    // ========== Security and DRM Errors ==========
    DRMViolation = 80,
    InvalidDRMConfig = 81,
    TokenExpired = 82,
    InvalidToken = 83,
    GeoblockingViolation = 84,
    EncryptionFailed = 85,
    DecryptionFailed = 86,
    
    // ========== Disaster Recovery Errors ==========
    BackupFailed = 90,
    BackupNotFound = 91,
    BackupCorrupted = 92,
    RestoreFailed = 93,
    RecoveryPlanNotFound = 94,
    RecoveryFailed = 95,
    
    // ========== General Errors ==========
    InvalidInput = 100,
    InternalError = 101,
    StorageError = 102,
    ConfigurationError = 103,
    OperationNotSupported = 104,
}
    
    // ========== Enhanced Streaming Errors ==========
    StreamingConfigNotFound = 110,
    InvalidStreamingProfile = 111,
    UnsupportedStreamingProtocol = 112,
    NetworkConditionError = 113,
    ManifestGenerationFailed = 114,
    QualityAdaptationFailed = 115,
    
    // ========== Enhanced Cost Optimization Errors ==========
    BudgetExceeded = 120,
    InvalidPricingModel = 121,
    CostCalculationFailed = 122,
    BudgetNotSet = 123,
    OptimizationFailed = 124,
    InsufficientCostData = 125,