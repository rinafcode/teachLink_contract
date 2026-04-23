use soroban_sdk::symbol_short;
use soroban_sdk::Symbol;

// Namespace for TeachLink contract storage keys
pub const NAMESPACE: Symbol = symbol_short!("teachlink");

// Storage keys for the bridge contract
pub const TOKEN: Symbol = symbol_short!("tl_token");
pub const VALIDATORS: Symbol = symbol_short!("tl_valids");
pub const MIN_VALIDATORS: Symbol = symbol_short!("tl_minval");
pub const NONCE: Symbol = symbol_short!("tl_nonce");
pub const BRIDGE_TXS: Symbol = symbol_short!("tl_brtxs");
pub const SUPPORTED_CHAINS: Symbol = symbol_short!("tl_chains");
pub const ADMIN: Symbol = symbol_short!("tl_admin");
pub const FEE_RECIPIENT: Symbol = symbol_short!("tl_feercp");
pub const BRIDGE_FEE: Symbol = symbol_short!("tl_brfee");
pub const BRIDGE_RETRY_COUNTS: Symbol = symbol_short!("tl_brrtrc");
pub const BRIDGE_LAST_RETRY: Symbol = symbol_short!("tl_brlstr");
pub const BRIDGE_FAILURES: Symbol = symbol_short!("tl_brfail");
pub const INTERFACE_VERSION: Symbol = symbol_short!("tl_ifver");
pub const MIN_COMPAT_INTERFACE_VERSION: Symbol = symbol_short!("tl_ifmin");

// ========== Advanced Bridge Storage Keys ==========

// BFT Consensus Storage
pub const VALIDATOR_INFO: Symbol = symbol_short!("tl_valinf");
pub const BRIDGE_PROPOSALS: Symbol = symbol_short!("tl_brprop");
pub const PROPOSAL_COUNTER: Symbol = symbol_short!("tl_propcnt");
pub const CONSENSUS_STATE: Symbol = symbol_short!("tl_consst");
pub const VALIDATOR_STAKES: Symbol = symbol_short!("tl_valstk");

// Slashing and Rewards Storage
pub const SLASHING_RECORDS: Symbol = symbol_short!("tl_slarec");
pub const VALIDATOR_REWARDS: Symbol = symbol_short!("tl_valrwd");
pub const SLASHING_COUNTER: Symbol = symbol_short!("tl_slacnt");

// Multi-Chain Support Storage
pub const CHAIN_CONFIGS: Symbol = symbol_short!("tl_chcfg");
pub const MULTI_CHAIN_ASSETS: Symbol = symbol_short!("tl_mcaset");
pub const ASSET_COUNTER: Symbol = symbol_short!("tl_asetcnt");

// Liquidity and AMM Storage
pub const LIQUIDITY_POOLS: Symbol = symbol_short!("tl_liqpol");
pub const LP_POSITIONS: Symbol = symbol_short!("tl_lppos");
pub const FEE_STRUCTURE: Symbol = symbol_short!("tl_feestr");

// Message Passing Storage
pub const CROSS_CHAIN_PACKETS: Symbol = symbol_short!("tl_ccpckt");
pub const PACKET_COUNTER: Symbol = symbol_short!("tl_pctcnt");
pub const MESSAGE_RECEIPTS: Symbol = symbol_short!("tl_msgrcp");
pub const PACKET_RETRY_COUNTS: Symbol = symbol_short!("tl_pctrtr");
pub const PACKET_LAST_RETRY: Symbol = symbol_short!("tl_pctlst");

// Emergency and Security Storage
pub const EMERGENCY_STATE: Symbol = symbol_short!("tl_emrsta");
pub const CIRCUIT_BREAKERS: Symbol = symbol_short!("tl_cirbrk");
pub const PAUSED_CHAINS: Symbol = symbol_short!("tl_puschn");

// Audit and Compliance Storage
pub const AUDIT_RECORDS: Symbol = symbol_short!("tl_audrec");
pub const AUDIT_COUNTER: Symbol = symbol_short!("tl_audcnt");
pub const COMPLIANCE_REPORTS: Symbol = symbol_short!("tl_comrep");

// Atomic Swap Storage
pub const ATOMIC_SWAPS: Symbol = symbol_short!("tl_atmswp");
pub const SWAP_COUNTER: Symbol = symbol_short!("tl_swpcnt");

// Analytics Storage
pub const BRIDGE_METRICS: Symbol = symbol_short!("tl_brmetr");
pub const CHAIN_METRICS: Symbol = symbol_short!("tl_chmetr");
pub const DAILY_VOLUMES: Symbol = symbol_short!("tl_dlyvol");

// Storage keys for the rewards system
pub const REWARDS_ADMIN: Symbol = symbol_short!("tl_rwadmn");
pub const REWARD_POOL: Symbol = symbol_short!("tl_rwpool");
pub const USER_REWARDS: Symbol = symbol_short!("tl_usrwrd");
pub const REWARD_RATES: Symbol = symbol_short!("tl_rwrate");
pub const TOTAL_REWARDS_ISSUED: Symbol = symbol_short!("tl_totrwd");
pub const ESCROW_COUNT: Symbol = symbol_short!("tl_esccnt");
pub const ESCROWS: Symbol = symbol_short!("tl_escrow");

// Storage keys for credit scoring
pub const CREDIT_SCORE: Symbol = symbol_short!("tl_crscr");
pub const COURSE_COMPLETIONS: Symbol = symbol_short!("tl_crscomp");
pub const CONTRIBUTIONS: Symbol = symbol_short!("tl_contrb");

// Storage keys for content tokenization
pub const TOKEN_COUNTER: Symbol = symbol_short!("tl_tokcnt");
pub const CONTENT_TOKENS: Symbol = symbol_short!("tl_cnttok");
pub const OWNERSHIP: Symbol = symbol_short!("tl_owners");
pub const PROVENANCE: Symbol = symbol_short!("tl_proven");
pub const OWNER_TOKENS: Symbol = symbol_short!("tl_owntok");

// Arbitration and insurance Storage
pub const ARBITRATORS: Symbol = symbol_short!("tl_arbitr");
pub const INSURANCE_POOL: Symbol = symbol_short!("tl_inspol");
pub const ESCROW_ANALYTICS: Symbol = symbol_short!("tl_escana");

// Notification System Storage
pub const NOTIFICATION_COUNTER: Symbol = symbol_short!("tl_notcnt");
pub const NOTIFICATION_LOGS: Symbol = symbol_short!("tl_notlog");
pub const NOTIFICATION_TRACKING: Symbol = symbol_short!("tl_nottrk");
pub const NOTIFICATION_PREFERENCES: Symbol = symbol_short!("tl_notprf");
pub const NOTIFICATION_TEMPLATES: Symbol = symbol_short!("tl_nottmp");
pub const SCHEDULED_NOTIFICATIONS: Symbol = symbol_short!("tl_notsch");
pub const USER_NOTIFICATION_SETTINGS: Symbol = symbol_short!("tl_notset");
pub const NOTIFICATION_BATCHES: Symbol = symbol_short!("tl_notbch");
pub const NOTIFICATION_AB_TESTS: Symbol = symbol_short!("tl_notabt");
pub const NOTIFICATION_COMPLIANCE: Symbol = symbol_short!("tl_notcmp");
pub const NOTIFICATION_RATE_LIMITS: Symbol = symbol_short!("tl_notrtl");
pub const NOTIFICATION_WEBHOOKS: Symbol = symbol_short!("tl_notweb");
pub const NOTIFICATION_FILTERS: Symbol = symbol_short!("tl_notflt");
pub const NOTIFICATION_SEGMENTS: Symbol = symbol_short!("tl_notseg");
pub const NOTIFICATION_CAMPAIGNS: Symbol = symbol_short!("tl_notcpg");
pub const NOTIFICATION_ANALYTICS: Symbol = symbol_short!("tl_notanl");

// Advanced Analytics & Reporting Storage (symbol_short! max 9 chars)
pub const REPORT_TEMPLATE_COUNTER: Symbol = symbol_short!("tl_rpttpc");
pub const REPORT_TEMPLATES: Symbol = symbol_short!("tl_rpttpl");
pub const REPORT_SCHEDULE_COUNTER: Symbol = symbol_short!("tl_rptscc");
pub const REPORT_SCHEDULES: Symbol = symbol_short!("tl_rptsch");
pub const REPORT_SNAPSHOT_COUNTER: Symbol = symbol_short!("tl_rptsnc");
pub const REPORT_SNAPSHOTS: Symbol = symbol_short!("tl_rptsnp");
pub const REPORT_USAGE: Symbol = symbol_short!("tl_rptuse");
pub const REPORT_COMMENT_COUNTER: Symbol = symbol_short!("tl_rptcmc");
pub const REPORT_COMMENTS: Symbol = symbol_short!("tl_rptcmt");
pub const ALERT_RULE_COUNTER: Symbol = symbol_short!("tl_alrcnt");
pub const ALERT_RULES: Symbol = symbol_short!("tl_alrrul");

// Backup and Disaster Recovery Storage (symbol_short! max 9 chars)
pub const BACKUP_COUNTER: Symbol = symbol_short!("tl_bakcnt");
pub const BACKUP_MANIFESTS: Symbol = symbol_short!("tl_bakmnf");
pub const BACKUP_SCHED_CNT: Symbol = symbol_short!("tl_bakscc");
pub const BACKUP_SCHEDULES: Symbol = symbol_short!("tl_baksch");
pub const RECOVERY_CNT: Symbol = symbol_short!("tl_reccnt");
pub const RECOVERY_RECORDS: Symbol = symbol_short!("tl_recrec");

// Performance optimization and caching (symbol_short! max 9 chars)
pub const PERF_CACHE: Symbol = symbol_short!("tl_perfc");
pub const PERF_TS: Symbol = symbol_short!("tl_perfts");

// Advanced UI/UX Storage (symbol_short! max 9 chars)
pub const ONBOARDING_STATUS: Symbol = symbol_short!("tl_onbrd");
pub const USER_FEEDBACK: Symbol = symbol_short!("tl_feedbk");
pub const UX_EXPERIMENTS: Symbol = symbol_short!("tl_uxexp");
pub const COMPONENT_CONFIG: Symbol = symbol_short!("tl_cmpcfg");

// Reentrancy guard locks
pub const BRIDGE_GUARD: Symbol = symbol_short!("tl_brgrd");
pub const REWARDS_GUARD: Symbol = symbol_short!("tl_rwgrd");
pub const SWAP_GUARD: Symbol = symbol_short!("tl_swgrd");
pub const INSURANCE_GUARD: Symbol = symbol_short!("tl_insgrd");

/// Returns all storage keys for collision detection
pub fn all_storage_keys() -> Vec<Symbol> {
    vec![
        NAMESPACE,
        TOKEN,
        VALIDATORS,
        MIN_VALIDATORS,
        NONCE,
        BRIDGE_TXS,
        SUPPORTED_CHAINS,
        ADMIN,
        FEE_RECIPIENT,
        BRIDGE_FEE,
        BRIDGE_RETRY_COUNTS,
        BRIDGE_LAST_RETRY,
        BRIDGE_FAILURES,
        INTERFACE_VERSION,
        MIN_COMPAT_INTERFACE_VERSION,
        VALIDATOR_INFO,
        BRIDGE_PROPOSALS,
        PROPOSAL_COUNTER,
        CONSENSUS_STATE,
        VALIDATOR_STAKES,
        SLASHING_RECORDS,
        VALIDATOR_REWARDS,
        SLASHING_COUNTER,
        CHAIN_CONFIGS,
        MULTI_CHAIN_ASSETS,
        ASSET_COUNTER,
        LIQUIDITY_POOLS,
        LP_POSITIONS,
        FEE_STRUCTURE,
        CROSS_CHAIN_PACKETS,
        PACKET_COUNTER,
        MESSAGE_RECEIPTS,
        PACKET_RETRY_COUNTS,
        PACKET_LAST_RETRY,
        EMERGENCY_STATE,
        CIRCUIT_BREAKERS,
        PAUSED_CHAINS,
        AUDIT_RECORDS,
        AUDIT_COUNTER,
        COMPLIANCE_REPORTS,
        ATOMIC_SWAPS,
        SWAP_COUNTER,
        BRIDGE_METRICS,
        CHAIN_METRICS,
        DAILY_VOLUMES,
        REWARDS_ADMIN,
        REWARD_POOL,
        USER_REWARDS,
        REWARD_RATES,
        TOTAL_REWARDS_ISSUED,
        ESCROW_COUNT,
        ESCROWS,
        CREDIT_SCORE,
        COURSE_COMPLETIONS,
        CONTRIBUTIONS,
        TOKEN_COUNTER,
        CONTENT_TOKENS,
        OWNERSHIP,
        PROVENANCE,
        OWNER_TOKENS,
        ARBITRATORS,
        INSURANCE_POOL,
        ESCROW_ANALYTICS,
        NOTIFICATION_COUNTER,
        NOTIFICATION_LOGS,
        NOTIFICATION_TRACKING,
        NOTIFICATION_PREFERENCES,
        NOTIFICATION_TEMPLATES,
        SCHEDULED_NOTIFICATIONS,
        USER_NOTIFICATION_SETTINGS,
        NOTIFICATION_BATCHES,
        NOTIFICATION_AB_TESTS,
        NOTIFICATION_COMPLIANCE,
        NOTIFICATION_RATE_LIMITS,
        NOTIFICATION_WEBHOOKS,
        NOTIFICATION_FILTERS,
        NOTIFICATION_SEGMENTS,
        NOTIFICATION_CAMPAIGNS,
        NOTIFICATION_ANALYTICS,
        REPORT_TEMPLATE_COUNTER,
        REPORT_TEMPLATES,
        REPORT_SCHEDULE_COUNTER,
        REPORT_SCHEDULES,
        REPORT_SNAPSHOT_COUNTER,
        REPORT_SNAPSHOTS,
        REPORT_USAGE,
        REPORT_COMMENT_COUNTER,
        REPORT_COMMENTS,
        ALERT_RULE_COUNTER,
        ALERT_RULES,
        BACKUP_COUNTER,
        BACKUP_MANIFESTS,
        BACKUP_SCHED_CNT,
        BACKUP_SCHEDULES,
        RECOVERY_CNT,
        RECOVERY_RECORDS,
        PERF_CACHE,
        PERF_TS,
        ONBOARDING_STATUS,
        USER_FEEDBACK,
        UX_EXPERIMENTS,
        COMPONENT_CONFIG,
        BRIDGE_GUARD,
        REWARDS_GUARD,
        SWAP_GUARD,
        INSURANCE_GUARD,
    ]
}

/// Detects potential key collisions by checking for duplicates
pub fn detect_key_collisions() -> Result<(), Symbol> {
    let keys = all_storage_keys();
    let mut sorted_keys = keys.clone();
    // Simple sort and check adjacent (since Symbol implements Ord in Soroban)
    sorted_keys.sort();
    for i in 1..sorted_keys.len() {
        if sorted_keys[i] == sorted_keys[i - 1] {
            return Err(sorted_keys[i]);
        }
    }
    Ok(())
}
