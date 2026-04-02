//! Event Querying and Filtering Utilities
//!
//! This module provides helper functions for querying and filtering events.

use soroban_sdk::{Address, Env, Vec, Symbol, symbol_short};

/// Event category types for filtering
#[derive(Clone, Debug, PartialEq)]
pub enum EventCategory {
    Bridge,
    Consensus,
    Slashing,
    Emergency,
    Escrow,
    Insurance,
    Reputation,
    Tokenization,
    Rewards,
    MultiChain,
    MessagePassing,
    CreditScore,
    Analytics,
    Backup,
    AtomicSwap,
    Audit,
    Performance,
}

/// Event query builder for flexible event filtering
pub struct EventQuery {
    env: Env,
    event_type: Option<Symbol>,
    address: Option<Address>,
    from_timestamp: Option<u64>,
    to_timestamp: Option<u64>,
}

impl EventQuery {
    /// Create a new event query
    pub fn new(env: &Env) -> Self {
        Self {
            env: env.clone(),
            event_type: None,
            address: None,
            from_timestamp: None,
            to_timestamp: None,
        }
    }

    /// Filter by event type
    pub fn with_type(mut self, event_type: Symbol) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Filter by address (any field containing an address)
    pub fn with_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }

    /// Filter by timestamp range (from)
    pub fn from_timestamp(mut self, timestamp: u64) -> Self {
        self.from_timestamp = Some(timestamp);
        self
    }

    /// Filter by timestamp range (to)
    pub fn to_timestamp(mut self, timestamp: u64) -> Self {
        self.to_timestamp = Some(timestamp);
        self
    }

    /// Execute the query and return matching events
    /// Note: This is a placeholder - actual event querying depends on Soroban's event API
    pub fn execute(self) -> Vec<u64> {
        // In a real implementation, this would query the event log
        // For now, return an empty vector as a placeholder
        Vec::new(&self.env)
    }
}

/// Helper functions for common event queries
pub mod queries {
    use super::*;
    
    /// Get all bridge-related events for a user
    pub fn get_bridge_events_for_user(env: &Env, user: Address) -> EventQuery {
        EventQuery::new(env)
            .with_address(user)
    }
    
    /// Get all escrow events for a specific escrow
    pub fn get_escrow_events(env: &Env, escrow_id: u64) -> EventQuery {
        // In practice, you'd filter by escrow_id field
        EventQuery::new(env)
    }
    
    /// Get all validator events
    pub fn get_validator_events(env: &Env, validator: Address) -> EventQuery {
        EventQuery::new(env)
            .with_address(validator)
    }
    
    /// Get events in a time range
    pub fn get_events_in_range(env: &Env, from: u64, to: u64) -> EventQuery {
        EventQuery::new(env)
            .from_timestamp(from)
            .to_timestamp(to)
    }
}

/// Event type symbols for querying
pub mod event_types {
    use soroban_sdk::{Symbol, symbol_short};
    
    // Bridge Events
    pub const BRIDGE_INITIATED: Symbol = symbol_short!("b_init");
    pub const BRIDGE_COMPLETED: Symbol = symbol_short!("b_comp");
    pub const BRIDGE_CANCELLED: Symbol = symbol_short!("b_can");
    pub const BRIDGE_FAILED: Symbol = symbol_short!("b_fail");
    pub const BRIDGE_RETRY: Symbol = symbol_short!("b_retry");
    pub const DEPOSIT: Symbol = symbol_short!("deposit");
    pub const RELEASE: Symbol = symbol_short!("release");
    pub const VALIDATOR_ADDED: Symbol = symbol_short!("v_add");
    pub const VALIDATOR_REMOVED: Symbol = symbol_short!("v_rem");
    pub const CHAIN_SUPPORTED: Symbol = symbol_short!("c_sup");
    pub const CHAIN_UNSUPPORTED: Symbol = symbol_short!("c_uns");
    pub const BRIDGE_FEE_UPDATED: Symbol = symbol_short!("fee_upd");
    pub const FEE_RECIPIENT_UPDATED: Symbol = symbol_short!("f_rec_u");
    pub const MIN_VALIDATORS_UPDATED: Symbol = symbol_short!("m_val_u");
    
    // Consensus Events
    pub const PROPOSAL_CREATED: Symbol = symbol_short!("p_creat");
    pub const PROPOSAL_VOTED: Symbol = symbol_short!("p_vote");
    pub const PROPOSAL_EXECUTED: Symbol = symbol_short!("p_exec");
    pub const VALIDATOR_REGISTERED: Symbol = symbol_short!("v_reg");
    pub const VALIDATOR_UNREGISTERED: Symbol = symbol_short!("v_unrg");
    
    // Slashing Events
    pub const VALIDATOR_SLASHED: Symbol = symbol_short!("v_slash");
    pub const VALIDATOR_REWARDED: Symbol = symbol_short!("v_rew");
    pub const STAKE_DEPOSITED: Symbol = symbol_short!("s_dep");
    pub const STAKE_WITHDRAWN: Symbol = symbol_short!("s_wit");
    pub const REWARD_POOL_FUNDED: Symbol = symbol_short!("r_pool");
    
    // Emergency Events
    pub const BRIDGE_PAUSED: Symbol = symbol_short!("pause");
    pub const BRIDGE_RESUMED: Symbol = symbol_short!("resume");
    pub const CIRCUIT_BREAKER_TRIGGERED: Symbol = symbol_short!("cb_trig");
    pub const CIRCUIT_BREAKER_INIT: Symbol = symbol_short!("cb_init");
    pub const CIRCUIT_BREAKER_RESET: Symbol = symbol_short!("cb_rst");
    pub const CIRCUIT_BREAKER_LIMITS_UPDATED: Symbol = symbol_short!("cb_l_u");
    
    // Escrow Events
    pub const ESCROW_CREATED: Symbol = symbol_short!("e_creat");
    pub const ESCROW_APPROVED: Symbol = symbol_short!("e_apr");
    pub const ESCROW_RELEASED: Symbol = symbol_short!("e_rel");
    pub const ESCROW_REFUNDED: Symbol = symbol_short!("e_ref");
    pub const ESCROW_CANCELLED: Symbol = symbol_short!("e_can");
    pub const ESCROW_DISPUTED: Symbol = symbol_short!("e_dis");
    pub const ESCROW_RESOLVED: Symbol = symbol_short!("e_res");
    
    // Insurance Events
    pub const INSURANCE_POOL_INIT: Symbol = symbol_short!("i_init");
    pub const INSURANCE_POOL_FUNDED: Symbol = symbol_short!("i_fund");
    pub const INSURANCE_PREMIUM_PAID: Symbol = symbol_short!("i_prem");
    pub const INSURANCE_CLAIM_PROCESSED: Symbol = symbol_short!("i_clm");
    
    // Reputation Events
    pub const PARTICIPATION_UPDATED: Symbol = symbol_short!("p_upd");
    pub const COURSE_PROGRESS_UPDATED: Symbol = symbol_short!("c_upd");
    pub const CONTRIBUTION_RATED: Symbol = symbol_short!("c_rate");
    
    // Tokenization Events
    pub const CONTENT_MINTED: Symbol = symbol_short!("mint");
    pub const OWNERSHIP_TRANSFERRED: Symbol = symbol_short!("trans");
    pub const PROVENANCE_RECORDED: Symbol = symbol_short!("proven");
    pub const METADATA_UPDATED: Symbol = symbol_short!("m_upd");
    pub const TRANSFERABILITY_UPDATED: Symbol = symbol_short!("t_upd");
    
    // Rewards Events
    pub const REWARD_ISSUED: Symbol = symbol_short!("r_iss");
    pub const REWARD_CLAIMED: Symbol = symbol_short!("r_clm");
    pub const REWARD_POOL_FUNDED_EXTERNAL: Symbol = symbol_short!("r_p_ext");
}

/// Check if an event belongs to a specific category
pub fn is_event_in_category(event_type: &Symbol, category: &EventCategory) -> bool {
    use event_types::*;
    
    match category {
        EventCategory::Bridge => {
            *event_type == BRIDGE_INITIATED ||
            *event_type == BRIDGE_COMPLETED ||
            *event_type == BRIDGE_CANCELLED ||
            *event_type == BRIDGE_FAILED ||
            *event_type == BRIDGE_RETRY ||
            *event_type == DEPOSIT ||
            *event_type == RELEASE ||
            *event_type == VALIDATOR_ADDED ||
            *event_type == VALIDATOR_REMOVED ||
            *event_type == CHAIN_SUPPORTED ||
            *event_type == CHAIN_UNSUPPORTED ||
            *event_type == BRIDGE_FEE_UPDATED ||
            *event_type == FEE_RECIPIENT_UPDATED ||
            *event_type == MIN_VALIDATORS_UPDATED
        }
        EventCategory::Consensus => {
            *event_type == PROPOSAL_CREATED ||
            *event_type == PROPOSAL_VOTED ||
            *event_type == PROPOSAL_EXECUTED ||
            *event_type == VALIDATOR_REGISTERED ||
            *event_type == VALIDATOR_UNREGISTERED
        }
        EventCategory::Slashing => {
            *event_type == VALIDATOR_SLASHED ||
            *event_type == VALIDATOR_REWARDED ||
            *event_type == STAKE_DEPOSITED ||
            *event_type == STAKE_WITHDRAWN ||
            *event_type == REWARD_POOL_FUNDED
        }
        EventCategory::Emergency => {
            *event_type == BRIDGE_PAUSED ||
            *event_type == BRIDGE_RESUMED ||
            *event_type == CIRCUIT_BREAKER_TRIGGERED ||
            *event_type == CIRCUIT_BREAKER_INIT ||
            *event_type == CIRCUIT_BREAKER_RESET ||
            *event_type == CIRCUIT_BREAKER_LIMITS_UPDATED
        }
        EventCategory::Escrow => {
            *event_type == ESCROW_CREATED ||
            *event_type == ESCROW_APPROVED ||
            *event_type == ESCROW_RELEASED ||
            *event_type == ESCROW_REFUNDED ||
            *event_type == ESCROW_CANCELLED ||
            *event_type == ESCROW_DISPUTED ||
            *event_type == ESCROW_RESOLVED
        }
        EventCategory::Insurance => {
            *event_type == INSURANCE_POOL_INIT ||
            *event_type == INSURANCE_POOL_FUNDED ||
            *event_type == INSURANCE_PREMIUM_PAID ||
            *event_type == INSURANCE_CLAIM_PROCESSED
        }
        EventCategory::Reputation => {
            *event_type == PARTICIPATION_UPDATED ||
            *event_type == COURSE_PROGRESS_UPDATED ||
            *event_type == CONTRIBUTION_RATED
        }
        EventCategory::Tokenization => {
            *event_type == CONTENT_MINTED ||
            *event_type == OWNERSHIP_TRANSFERRED ||
            *event_type == PROVENANCE_RECORDED ||
            *event_type == METADATA_UPDATED ||
            *event_type == TRANSFERABILITY_UPDATED
        }
        EventCategory::Rewards => {
            *event_type == REWARD_ISSUED ||
            *event_type == REWARD_CLAIMED ||
            *event_type == REWARD_POOL_FUNDED_EXTERNAL
        }
        // Other categories can be added as needed
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_query_builder() {
        let env = Env::default();
        let query = EventQuery::new(&env)
            .with_type(event_types::BRIDGE_INITIATED)
            .from_timestamp(1000)
            .to_timestamp(2000);
        
        // Verify query is built correctly
        assert!(query.event_type.is_some());
        assert!(query.from_timestamp.is_some());
        assert!(query.to_timestamp.is_some());
    }
    
    #[test]
    fn test_categorization() {
        use event_types::*;
        
        assert!(is_event_in_category(&BRIDGE_INITIATED, &EventCategory::Bridge));
        assert!(is_event_in_category(&ESCROW_CREATED, &EventCategory::Escrow));
        assert!(is_event_in_category(&VALIDATOR_SLASHED, &EventCategory::Slashing));
        assert!(is_event_in_category(&INSURANCE_CLAIM_PROCESSED, &EventCategory::Insurance));
        
        assert!(!is_event_in_category(&BRIDGE_INITIATED, &EventCategory::Escrow));
        assert!(!is_event_in_category(&ESCROW_CREATED, &EventCategory::Bridge));
    }
}
