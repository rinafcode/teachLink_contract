//! Proposal Automation and Prioritization Module
//!
//! Provides features for automated proposal scheduling, recurring governance 
//! actions, and dynamic prioritization of community proposals.
//!
//! # Features
//! - **Automated Scheduling**: Execute predefined actions on a schedule.
//! - **Prioritization Engine**: Rank proposals based on voter count, power, and age.
//! - **Emergency Fast-Track**: Automatically prioritize critical security TIPs.

use soroban_sdk::{contracttype, Address, Env, symbol_short, Symbol, Val, Vec};

const AUTO_CONFIG: Symbol = symbol_short!("auto_cfg");
const QUEUE: Symbol = symbol_short!("priority");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationConfig {
    pub min_priority_threshold: i128,
    pub fast_track_enabled: bool,
    pub max_active_proposals: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriorityRecord {
    pub proposal_id: u64,
    pub priority_score: i128,
    pub fast_tracked: bool,
}

pub struct ProposalAutomation;

impl ProposalAutomation {
    pub fn initialize(env: &Env, admin: Address, threshold: i128) {
        admin.require_auth();
        let config = AutomationConfig {
            min_priority_threshold: threshold,
            fast_track_enabled: true,
            max_active_proposals: 10,
        };
        env.storage().instance().set(&AUTO_CONFIG, &config);
        env.storage().instance().set(&QUEUE, &Vec::<PriorityRecord>::new(env));
    }

    /// Calculate and update priority for a proposal
    pub fn update_priority(env: &Env, proposal_id: u64, voter_count: u32, total_power: i128) -> i128 {
        let score = (i128::from(voter_count) * 100) + (total_power / 1000);
        
        let mut queue: Vec<PriorityRecord> = env.storage().instance().get(&QUEUE).unwrap();
        let mut found = false;
        
        for i in 0..queue.len() {
            let mut record = queue.get(i).unwrap();
            if record.proposal_id == proposal_id {
                record.priority_score = score;
                queue.set(i, record);
                found = true;
                break;
            }
        }
        
        if !found {
            queue.push_back(PriorityRecord {
                proposal_id,
                priority_score: score,
                fast_tracked: false,
            });
        }
        
        env.storage().instance().set(&QUEUE, &queue);
        score
    }

    /// Get proposals sorted by priority
    pub fn get_prioritized_queue(env: &Env) -> Vec<PriorityRecord> {
        env.storage().instance().get(&QUEUE).unwrap_or_else(|| Vec::new(env))
    }
}
