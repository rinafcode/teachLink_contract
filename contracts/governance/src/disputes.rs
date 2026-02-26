//! Dispute Resolution and Appeals Module
//!
//! Provides mechanisms for challenging governance decisions through
//! formal disputes and appeals. This ensures accountability and
//! protects against governance attacks or unfair outcomes.
//!
//! # Dispute Lifecycle
//!
//! 1. A participant files a dispute against a proposal outcome
//! 2. The dispute enters review period
//! 3. Community or admin resolves the dispute
//! 4. If dismissed, the original outcome stands
//! 5. If upheld, the proposal may be reversed or re-voted
//!
//! # Appeal Process
//!
//! After a dispute is resolved, the disputant can file an appeal
//! within the appeal window. Appeals are reviewed by admin or
//! governance council.

use soroban_sdk::{Address, Bytes, Env};

use crate::events;
use crate::storage::{APPEALS, DISPUTES, DISPUTE_COUNT};
use crate::types::{Appeal, Dispute, DisputeKey, DisputeStatus};

/// Default dispute resolution deadline (7 days)
const DEFAULT_RESOLUTION_PERIOD: u64 = 604800;

/// Appeal window after dispute resolution (3 days)
const APPEAL_WINDOW: u64 = 259200;

pub struct DisputeResolution;

impl DisputeResolution {
    /// File a dispute against a proposal outcome
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `disputant` - Address filing the dispute
    /// * `proposal_id` - Proposal being disputed
    /// * `reason` - Reason for the dispute
    ///
    /// # Returns
    /// The dispute ID
    pub fn file_dispute(env: &Env, disputant: Address, proposal_id: u64, reason: Bytes) -> u64 {
        disputant.require_auth();

        assert!(
            !reason.is_empty(),
            "ERR_EMPTY_REASON: Dispute reason cannot be empty"
        );

        let now = env.ledger().timestamp();

        let mut dispute_count: u64 = env.storage().instance().get(&DISPUTE_COUNT).unwrap_or(0);
        dispute_count += 1;

        let dispute = Dispute {
            id: dispute_count,
            proposal_id,
            disputant: disputant.clone(),
            reason,
            status: DisputeStatus::Open,
            created_at: now,
            resolution_deadline: now + DEFAULT_RESOLUTION_PERIOD,
            resolution: None,
            resolver: None,
            for_votes: 0,
            against_votes: 0,
        };

        let dispute_key = DisputeKey {
            dispute_id: dispute_count,
        };

        env.storage()
            .persistent()
            .set(&(DISPUTES, dispute_key), &dispute);
        env.storage().instance().set(&DISPUTE_COUNT, &dispute_count);

        events::dispute_filed(env, dispute_count, proposal_id, &disputant);

        dispute_count
    }

    /// Vote on a dispute (community resolution)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `dispute_id` - Dispute to vote on
    /// * `voter` - Address casting the vote
    /// * `support` - true = uphold dispute, false = dismiss
    /// * `power` - Voting power to apply
    pub fn vote_on_dispute(env: &Env, dispute_id: u64, voter: Address, support: bool, power: i128) {
        voter.require_auth();

        let dispute_key = DisputeKey { dispute_id };

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&(DISPUTES, dispute_key.clone()))
            .expect("ERR_DISPUTE_NOT_FOUND: Dispute does not exist");

        assert!(
            dispute.status == DisputeStatus::Open || dispute.status == DisputeStatus::UnderReview,
            "ERR_DISPUTE_NOT_VOTEABLE: Dispute is not open for voting"
        );

        let now = env.ledger().timestamp();
        assert!(
            now <= dispute.resolution_deadline,
            "ERR_DISPUTE_DEADLINE_PASSED: Resolution deadline has passed"
        );

        if support {
            dispute.for_votes += power;
        } else {
            dispute.against_votes += power;
        }

        // Move to under review if first vote
        if dispute.status == DisputeStatus::Open {
            dispute.status = DisputeStatus::UnderReview;
        }

        env.storage()
            .persistent()
            .set(&(DISPUTES, dispute_key), &dispute);
    }

    /// Resolve a dispute (admin or after voting)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `dispute_id` - Dispute to resolve
    /// * `resolver` - Address resolving the dispute
    /// * `upheld` - Whether the dispute is upheld
    /// * `resolution` - Resolution description
    pub fn resolve_dispute(
        env: &Env,
        dispute_id: u64,
        resolver: Address,
        upheld: bool,
        resolution: Bytes,
    ) {
        resolver.require_auth();

        let dispute_key = DisputeKey { dispute_id };

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&(DISPUTES, dispute_key.clone()))
            .expect("ERR_DISPUTE_NOT_FOUND: Dispute does not exist");

        assert!(
            dispute.status != DisputeStatus::Resolved && dispute.status != DisputeStatus::Dismissed,
            "ERR_DISPUTE_ALREADY_RESOLVED: Dispute has already been resolved"
        );

        dispute.status = if upheld {
            DisputeStatus::Resolved
        } else {
            DisputeStatus::Dismissed
        };
        dispute.resolution = Some(resolution);
        dispute.resolver = Some(resolver.clone());

        env.storage()
            .persistent()
            .set(&(DISPUTES, dispute_key), &dispute);

        events::dispute_resolved(env, dispute_id, &resolver, upheld);
    }

    /// File an appeal against a dispute resolution
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `dispute_id` - Dispute to appeal
    /// * `appellant` - Address filing the appeal
    /// * `reason` - Reason for the appeal
    pub fn file_appeal(env: &Env, dispute_id: u64, appellant: Address, reason: Bytes) {
        appellant.require_auth();

        let dispute_key = DisputeKey { dispute_id };

        let dispute: Dispute = env
            .storage()
            .persistent()
            .get(&(DISPUTES, dispute_key.clone()))
            .expect("ERR_DISPUTE_NOT_FOUND: Dispute does not exist");

        assert!(
            dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Dismissed,
            "ERR_DISPUTE_NOT_RESOLVED: Dispute must be resolved before appeal"
        );

        let now = env.ledger().timestamp();
        // Appeal must be within the appeal window
        assert!(
            now <= dispute.resolution_deadline + APPEAL_WINDOW,
            "ERR_APPEAL_DEADLINE_PASSED: Appeal window has closed"
        );

        let appeal = Appeal {
            dispute_id,
            appellant: appellant.clone(),
            reason,
            created_at: now,
            granted: false,
        };

        env.storage()
            .persistent()
            .set(&(APPEALS, dispute_id), &appeal);

        // Update dispute status
        let mut updated_dispute = dispute;
        updated_dispute.status = DisputeStatus::Appealed;
        env.storage()
            .persistent()
            .set(&(DISPUTES, dispute_key), &updated_dispute);

        events::appeal_filed(env, dispute_id, &appellant);
    }

    /// Resolve an appeal (admin only)
    pub fn resolve_appeal(env: &Env, dispute_id: u64, admin: Address, granted: bool) {
        admin.require_auth();

        let mut appeal: Appeal = env
            .storage()
            .persistent()
            .get(&(APPEALS, dispute_id))
            .expect("ERR_APPEAL_NOT_FOUND: Appeal does not exist");

        appeal.granted = granted;
        env.storage()
            .persistent()
            .set(&(APPEALS, dispute_id), &appeal);

        // If appeal granted, update dispute status back to resolved (in favor)
        if granted {
            let dispute_key = DisputeKey { dispute_id };
            if let Some(mut dispute) = env
                .storage()
                .persistent()
                .get::<_, Dispute>(&(DISPUTES, dispute_key.clone()))
            {
                dispute.status = DisputeStatus::Resolved;
                env.storage()
                    .persistent()
                    .set(&(DISPUTES, dispute_key), &dispute);
            }
        }

        events::appeal_resolved(env, dispute_id, &admin, granted);
    }

    // ========== View Functions ==========

    /// Get a dispute by ID
    pub fn get_dispute(env: &Env, dispute_id: u64) -> Option<Dispute> {
        let dispute_key = DisputeKey { dispute_id };
        env.storage().persistent().get(&(DISPUTES, dispute_key))
    }

    /// Get an appeal by dispute ID
    pub fn get_appeal(env: &Env, dispute_id: u64) -> Option<Appeal> {
        env.storage().persistent().get(&(APPEALS, dispute_id))
    }

    /// Get total dispute count
    pub fn get_dispute_count(env: &Env) -> u64 {
        env.storage().instance().get(&DISPUTE_COUNT).unwrap_or(0)
    }
}
