use soroban_sdk::{symbol_short, Symbol, Vec, Env};

pub const NAMESPACE: Symbol = symbol_short!("teachlnk");

// Core
pub const TOKEN: Symbol = symbol_short!("tl_token");
pub const VALIDATORS: Symbol = symbol_short!("tl_valid");
pub const MIN_VALIDATORS: Symbol = symbol_short!("tl_minvl");
pub const NONCE: Symbol = symbol_short!("tl_nonce");
pub const BRIDGE_TXS: Symbol = symbol_short!("tl_brtxs");
pub const SUPPORTED_CHAINS: Symbol = symbol_short!("tl_chain");
pub const ADMIN: Symbol = symbol_short!("tl_admin");
pub const FEE_RECIPIENT: Symbol = symbol_short!("tl_feerc");
pub const BRIDGE_FEE: Symbol = symbol_short!("tl_brfee");

// BFT
pub const VALIDATOR_INFO: Symbol = symbol_short!("tl_valif");
pub const BRIDGE_PROPOSALS: Symbol = symbol_short!("tl_brprp");
pub const PROPOSAL_COUNTER: Symbol = symbol_short!("tl_propct"); // ✅ FIXED
pub const CONSENSUS_STATE: Symbol = symbol_short!("tl_const");
pub const VALIDATOR_STAKES: Symbol = symbol_short!("tl_vlstk");

// Slashing
pub const SLASHING_RECORDS: Symbol = symbol_short!("tl_slrec");
pub const VALIDATOR_REWARDS: Symbol = symbol_short!("tl_vlrwd");
pub const SLASHING_COUNTER: Symbol = symbol_short!("tl_slcnt");

// Multi-chain
pub const CHAIN_CONFIGS: Symbol = symbol_short!("tl_chcfg");
pub const MULTI_CHAIN_ASSETS: Symbol = symbol_short!("tl_mcast");
pub const ASSET_COUNTER: Symbol = symbol_short!("tl_asetct"); // ✅ FIXED

// Analytics / credit
pub const CREDIT_SCORE: Symbol = symbol_short!("tl_crscr");
pub const COURSE_COMPLETIONS: Symbol = symbol_short!("tl_crsco"); // ✅ FIXED

// Misc
pub const TOKEN_COUNTER: Symbol = symbol_short!("tl_tokct");
pub const CONTENT_TOKENS: Symbol = symbol_short!("tl_cnttk");

/// ✅ Soroban-safe key list
pub fn all_storage_keys(env: &Env) -> Vec<Symbol> {
    let mut keys = Vec::new(env);

    keys.push_back(NAMESPACE);
    keys.push_back(TOKEN);
    keys.push_back(VALIDATORS);
    keys.push_back(MIN_VALIDATORS);
    keys.push_back(NONCE);
    keys.push_back(BRIDGE_TXS);
    keys.push_back(SUPPORTED_CHAINS);
    keys.push_back(ADMIN);
    keys.push_back(FEE_RECIPIENT);
    keys.push_back(BRIDGE_FEE);

    keys.push_back(VALIDATOR_INFO);
    keys.push_back(BRIDGE_PROPOSALS);
    keys.push_back(PROPOSAL_COUNTER);
    keys.push_back(CONSENSUS_STATE);
    keys.push_back(VALIDATOR_STAKES);

    keys.push_back(SLASHING_RECORDS);
    keys.push_back(VALIDATOR_REWARDS);
    keys.push_back(SLASHING_COUNTER);

    keys.push_back(CHAIN_CONFIGS);
    keys.push_back(MULTI_CHAIN_ASSETS);
    keys.push_back(ASSET_COUNTER);

    keys.push_back(CREDIT_SCORE);
    keys.push_back(COURSE_COMPLETIONS);

    keys.push_back(TOKEN_COUNTER);
    keys.push_back(CONTENT_TOKENS);

    keys
}

/// ✅ No sorting (Soroban-safe)
pub fn detect_key_collisions(env: &Env) -> Result<(), Symbol> {
    let keys = all_storage_keys(env);

    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            if keys.get(i).unwrap() == keys.get(j).unwrap() {
                return Err(keys.get(i).unwrap());
            }
        }
    }

    Ok(())
}