use soroban_sdk::symbol_short;
use soroban_sdk::Symbol;

// Storage keys for the bridge contract
pub const TOKEN: Symbol = symbol_short!("token");
pub const VALIDATORS: Symbol = symbol_short!("validtor");
pub const MIN_VALIDATORS: Symbol = symbol_short!("min_valid");
pub const NONCE: Symbol = symbol_short!("nonce");
pub const BRIDGE_TXS: Symbol = symbol_short!("bridge_tx");
pub const SUPPORTED_CHAINS: Symbol = symbol_short!("chains");
pub const ADMIN: Symbol = symbol_short!("admin");
pub const FEE_RECIPIENT: Symbol = symbol_short!("fee_rcpt");
pub const BRIDGE_FEE: Symbol = symbol_short!("bridgefee");

// ========== Advanced Bridge Storage Keys ==========

// BFT Consensus Storage
pub const VALIDATOR_INFO: Symbol = symbol_short!("val_info");
pub const BRIDGE_PROPOSALS: Symbol = symbol_short!("proposals");
pub const PROPOSAL_COUNTER: Symbol = symbol_short!("prop_cnt");
pub const CONSENSUS_STATE: Symbol = symbol_short!("cons_st");
pub const VALIDATOR_STAKES: Symbol = symbol_short!("val_stake");

// Slashing and Rewards Storage
pub const SLASHING_RECORDS: Symbol = symbol_short!("slash_rec");
pub const VALIDATOR_REWARDS: Symbol = symbol_short!("val_rwds");
pub const SLASHING_COUNTER: Symbol = symbol_short!("slash_cnt");

// Multi-Chain Support Storage
pub const CHAIN_CONFIGS: Symbol = symbol_short!("chain_cfg");
pub const MULTI_CHAIN_ASSETS: Symbol = symbol_short!("mc_assets");
pub const ASSET_COUNTER: Symbol = symbol_short!("asset_cnt");

// Liquidity and AMM Storage
pub const LIQUIDITY_POOLS: Symbol = symbol_short!("liq_pools");
pub const LP_POSITIONS: Symbol = symbol_short!("lp_pos");
pub const FEE_STRUCTURE: Symbol = symbol_short!("fee_struc");

// Message Passing Storage
pub const CROSS_CHAIN_PACKETS: Symbol = symbol_short!("packets");
pub const PACKET_COUNTER: Symbol = symbol_short!("pkt_cnt");
pub const MESSAGE_RECEIPTS: Symbol = symbol_short!("receipts");

// Emergency and Security Storage
pub const EMERGENCY_STATE: Symbol = symbol_short!("emergency");
pub const CIRCUIT_BREAKERS: Symbol = symbol_short!("circ_brk");
pub const PAUSED_CHAINS: Symbol = symbol_short!("paused_ch");

// Audit and Compliance Storage
pub const AUDIT_RECORDS: Symbol = symbol_short!("audit_rec");
pub const AUDIT_COUNTER: Symbol = symbol_short!("audit_cnt");
pub const COMPLIANCE_REPORTS: Symbol = symbol_short!("compl_rep");

// Atomic Swap Storage
pub const ATOMIC_SWAPS: Symbol = symbol_short!("swaps");
pub const SWAP_COUNTER: Symbol = symbol_short!("swap_cnt");

// Analytics Storage
pub const BRIDGE_METRICS: Symbol = symbol_short!("metrics");
pub const CHAIN_METRICS: Symbol = symbol_short!("ch_mets");
pub const DAILY_VOLUMES: Symbol = symbol_short!("daily_vol");

// Storage keys for the rewards system
pub const REWARDS_ADMIN: Symbol = symbol_short!("rwd_admin");
pub const REWARD_POOL: Symbol = symbol_short!("rwd_pool");
pub const USER_REWARDS: Symbol = symbol_short!("usr_rwds");
pub const REWARD_RATES: Symbol = symbol_short!("rwd_rates");
pub const TOTAL_REWARDS_ISSUED: Symbol = symbol_short!("tot_rwds");
pub const ESCROW_COUNT: Symbol = symbol_short!("esc_ct");
pub const ESCROWS: Symbol = symbol_short!("escrows");

// Storage keys for credit scoring
pub const CREDIT_SCORE: Symbol = symbol_short!("score");
pub const COURSE_COMPLETIONS: Symbol = symbol_short!("courses");
pub const CONTRIBUTIONS: Symbol = symbol_short!("contribs");

// Storage keys for content tokenization
pub const TOKEN_COUNTER: Symbol = symbol_short!("tok_cnt");
pub const CONTENT_TOKENS: Symbol = symbol_short!("cnt_tok");
pub const OWNERSHIP: Symbol = symbol_short!("owner");
pub const PROVENANCE: Symbol = symbol_short!("prov");
pub const OWNER_TOKENS: Symbol = symbol_short!("own_tok");

// Arbitration and insurance Storage
pub const ARBITRATORS: Symbol = symbol_short!("arbs");
pub const INSURANCE_POOL: Symbol = symbol_short!("ins_pool");
pub const ESCROW_ANALYTICS: Symbol = symbol_short!("esc_an");

// Notification System Storage
pub const NOTIFICATION_COUNTER: Symbol = symbol_short!("notif_cnt");
pub const NOTIFICATION_LOGS: Symbol = symbol_short!("notif_log");
pub const NOTIFICATION_TRACKING: Symbol = symbol_short!("notif_trk");
pub const NOTIFICATION_PREFERENCES: Symbol = symbol_short!("notif_prf");
pub const NOTIFICATION_TEMPLATES: Symbol = symbol_short!("notif_tmp");
pub const SCHEDULED_NOTIFICATIONS: Symbol = symbol_short!("notif_sch");
pub const USER_NOTIFICATION_SETTINGS: Symbol = symbol_short!("notif_set");
pub const NOTIFICATION_BATCHES: Symbol = symbol_short!("notif_bch");
pub const NOTIFICATION_AB_TESTS: Symbol = symbol_short!("notif_ab");
pub const NOTIFICATION_COMPLIANCE: Symbol = symbol_short!("notif_cmp");
pub const NOTIFICATION_RATE_LIMITS: Symbol = symbol_short!("notif_rt");
pub const NOTIFICATION_WEBHOOKS: Symbol = symbol_short!("notif_web");
pub const NOTIFICATION_FILTERS: Symbol = symbol_short!("notif_flt");
pub const NOTIFICATION_SEGMENTS: Symbol = symbol_short!("notif_seg");
pub const NOTIFICATION_CAMPAIGNS: Symbol = symbol_short!("notif_cpg");
pub const NOTIFICATION_ANALYTICS: Symbol = symbol_short!("notif_anl");

// Advanced Analytics & Reporting Storage (symbol_short! max 9 chars)
pub const REPORT_TEMPLATE_COUNTER: Symbol = symbol_short!("rpt_tplcn");
pub const REPORT_TEMPLATES: Symbol = symbol_short!("rpt_tpl");
pub const REPORT_SCHEDULE_COUNTER: Symbol = symbol_short!("rpt_schcn");
pub const REPORT_SCHEDULES: Symbol = symbol_short!("rpt_sch");
pub const REPORT_SNAPSHOT_COUNTER: Symbol = symbol_short!("rpt_snpcn");
pub const REPORT_SNAPSHOTS: Symbol = symbol_short!("rpt_snp");
pub const REPORT_USAGE: Symbol = symbol_short!("rpt_use");
pub const REPORT_COMMENT_COUNTER: Symbol = symbol_short!("rpt_cmtcn");
pub const REPORT_COMMENTS: Symbol = symbol_short!("rpt_cmt");
pub const ALERT_RULE_COUNTER: Symbol = symbol_short!("alrt_cnt");
pub const ALERT_RULES: Symbol = symbol_short!("alrt_ruls");

// Backup and Disaster Recovery Storage (symbol_short! max 9 chars)
pub const BACKUP_COUNTER: Symbol = symbol_short!("bak_cnt");
pub const BACKUP_MANIFESTS: Symbol = symbol_short!("bak_mnf");
pub const BACKUP_SCHED_CNT: Symbol = symbol_short!("bak_scc");
pub const BACKUP_SCHEDULES: Symbol = symbol_short!("bak_sch");
pub const RECOVERY_CNT: Symbol = symbol_short!("rec_cnt");
pub const RECOVERY_RECORDS: Symbol = symbol_short!("rec_rec");

// Performance optimization and caching (symbol_short! max 9 chars)
pub const PERF_CACHE: Symbol = symbol_short!("perf_cach");
pub const PERF_TS: Symbol = symbol_short!("perf_ts");
