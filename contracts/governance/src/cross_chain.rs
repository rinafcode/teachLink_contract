//! Cross-Chain Governance Coordination Module
//!
//! Enables governance coordination across multiple chains by tracking
//! cross-chain proposals, recording external governance actions, and
//! enabling synchronized multi-chain voting.
//!
//! # How It Works
//!
//! 1. A cross-chain proposal is registered with references to external chain IDs
//! 2. External vote results can be recorded and aggregated
//! 3. Cross-chain quorum combines local and external votes
//! 4. Final outcomes are synchronized across chains
//!
//! Since Soroban contracts cannot directly call other chains, this module
//! uses a relay/oracle pattern: trusted relayers submit external chain
//! state, which is verified and aggregated on-chain.

use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol};

// use crate::events;

/// Storage key for cross-chain proposals
const XCHAIN_PROPOSALS: Symbol = symbol_short!("xc_prop");

/// Storage key for cross-chain proposal count
const XCHAIN_COUNT: Symbol = symbol_short!("xc_cnt");

/// Storage key for registered chain info
const CHAIN_REGISTRY: Symbol = symbol_short!("chains");

/// Storage key for relayer addresses
const RELAYERS: Symbol = symbol_short!("relayers");

/// Represents a registered external chain
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainInfo {
    /// Unique chain identifier (e.g., "ethereum", "polygon", "solana")
    pub chain_id: Bytes,
    /// Human-readable chain name
    pub name: Bytes,
    /// Whether the chain is active for governance
    pub active: bool,
    /// Weight assigned to this chain's votes (basis points, 10000 = 1x)
    pub vote_weight_bps: u32,
    /// Timestamp of registration
    pub registered_at: u64,
}

/// Cross-chain proposal that aggregates votes from multiple chains
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainProposal {
    /// Unique cross-chain proposal ID
    pub id: u64,
    /// Local proposal ID this maps to
    pub local_proposal_id: u64,
    /// Chain ID where proposal originated
    pub origin_chain: Bytes,
    /// Creator of the cross-chain proposal
    pub creator: Address,
    /// Aggregated for votes from all chains
    pub total_for_votes: i128,
    /// Aggregated against votes from all chains
    pub total_against_votes: i128,
    /// Aggregated abstain votes from all chains
    pub total_abstain_votes: i128,
    /// Number of chains that have reported results
    pub chains_reported: u32,
    /// Total number of chains participating
    pub total_chains: u32,
    /// Whether cross-chain quorum is met
    pub quorum_met: bool,
    /// Timestamp of creation
    pub created_at: u64,
    /// Whether the proposal has been finalized
    pub finalized: bool,
}

/// External chain vote report submitted by a relayer
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalVoteReport {
    /// Cross-chain proposal ID
    pub xc_proposal_id: u64,
    /// Chain the report is from
    pub chain_id: Bytes,
    /// For votes on that chain
    pub for_votes: i128,
    /// Against votes on that chain
    pub against_votes: i128,
    /// Abstain votes on that chain
    pub abstain_votes: i128,
    /// Relayer that submitted the report
    pub relayer: Address,
    /// Timestamp of submission
    pub submitted_at: u64,
    /// Whether report has been verified
    pub verified: bool,
}

pub struct CrossChainGovernance;

impl CrossChainGovernance {
    /// Register a new chain for cross-chain governance (admin only)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - Admin address (must authorize)
    /// * `chain_id` - Unique chain identifier
    /// * `name` - Human-readable chain name
    /// * `vote_weight_bps` - How much weight this chain's votes carry (basis points)
    pub fn register_chain(
        env: &Env,
        admin: Address,
        chain_id: Bytes,
        name: Bytes,
        vote_weight_bps: u32,
    ) {
        admin.require_auth();

        assert!(
            !chain_id.is_empty(),
            "ERR_EMPTY_CHAIN_ID: Chain ID cannot be empty"
        );

        assert!(
            vote_weight_bps > 0 && vote_weight_bps <= 10000,
            "ERR_INVALID_WEIGHT: Vote weight must be between 1 and 10000"
        );

        let chain_info = ChainInfo {
            chain_id: chain_id.clone(),
            name,
            active: true,
            vote_weight_bps,
            registered_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&(CHAIN_REGISTRY, chain_id), &chain_info);
    }

    /// Register a trusted relayer address (admin only)
    pub fn register_relayer(env: &Env, admin: Address, relayer: Address) {
        admin.require_auth();

        env.storage()
            .persistent()
            .set(&(RELAYERS, relayer.clone()), &true);
    }

    /// Check if an address is a registered relayer
    pub fn is_relayer(env: &Env, address: &Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&(RELAYERS, address.clone()))
            .unwrap_or(false)
    }

    /// Create a cross-chain proposal linking to a local proposal
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - Address creating the cross-chain proposal
    /// * `local_proposal_id` - Local proposal this maps to
    /// * `origin_chain` - Chain where proposal originated
    /// * `total_chains` - Total number of chains participating
    ///
    /// # Returns
    /// The cross-chain proposal ID
    pub fn create_cross_chain_proposal(
        env: &Env,
        creator: Address,
        local_proposal_id: u64,
        origin_chain: Bytes,
        total_chains: u32,
    ) -> u64 {
        creator.require_auth();

        assert!(
            total_chains > 0,
            "ERR_INVALID_CHAINS: Must have at least one participating chain"
        );

        let mut count: u64 = env.storage().instance().get(&XCHAIN_COUNT).unwrap_or(0);
        count += 1;

        let xc_proposal = CrossChainProposal {
            id: count,
            local_proposal_id,
            origin_chain,
            creator: creator.clone(),
            total_for_votes: 0,
            total_against_votes: 0,
            total_abstain_votes: 0,
            chains_reported: 0,
            total_chains,
            quorum_met: false,
            created_at: env.ledger().timestamp(),
            finalized: false,
        };

        env.storage()
            .persistent()
            .set(&(XCHAIN_PROPOSALS, count), &xc_proposal);
        env.storage().instance().set(&XCHAIN_COUNT, &count);

        count
    }

    /// Submit external chain vote results (relayer only)
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `relayer` - Trusted relayer submitting the report
    /// * `xc_proposal_id` - Cross-chain proposal ID
    /// * `chain_id` - Chain the votes are from
    /// * `for_votes` - For votes on that chain
    /// * `against_votes` - Against votes on that chain
    /// * `abstain_votes` - Abstain votes on that chain
    pub fn submit_external_votes(
        env: &Env,
        relayer: Address,
        xc_proposal_id: u64,
        chain_id: Bytes,
        for_votes: i128,
        against_votes: i128,
        abstain_votes: i128,
    ) {
        relayer.require_auth();

        assert!(
            Self::is_relayer(env, &relayer),
            "ERR_NOT_RELAYER: Only registered relayers can submit external votes"
        );

        let mut xc_proposal: CrossChainProposal = env
            .storage()
            .persistent()
            .get(&(XCHAIN_PROPOSALS, xc_proposal_id))
            .expect("ERR_XC_PROPOSAL_NOT_FOUND: Cross-chain proposal not found");

        assert!(
            !xc_proposal.finalized,
            "ERR_XC_ALREADY_FINALIZED: Cross-chain proposal already finalized"
        );

        // Get chain weight
        let chain_info: ChainInfo = env
            .storage()
            .persistent()
            .get(&(CHAIN_REGISTRY, chain_id.clone()))
            .expect("ERR_CHAIN_NOT_FOUND: Chain not registered");

        assert!(chain_info.active, "ERR_CHAIN_INACTIVE: Chain is not active");

        // Apply weight to votes
        let weight = i128::from(chain_info.vote_weight_bps);
        let weighted_for = for_votes * weight / 10000;
        let weighted_against = against_votes * weight / 10000;
        let weighted_abstain = abstain_votes * weight / 10000;

        // Aggregate votes
        xc_proposal.total_for_votes += weighted_for;
        xc_proposal.total_against_votes += weighted_against;
        xc_proposal.total_abstain_votes += weighted_abstain;
        xc_proposal.chains_reported += 1;

        // Check if quorum is met (all chains reported)
        if xc_proposal.chains_reported >= xc_proposal.total_chains {
            xc_proposal.quorum_met = true;
        }

        // Store the report
        let report = ExternalVoteReport {
            xc_proposal_id,
            chain_id: chain_id.clone(),
            for_votes,
            against_votes,
            abstain_votes,
            relayer: relayer.clone(),
            submitted_at: env.ledger().timestamp(),
            verified: true,
        };

        env.storage()
            .persistent()
            .set(&(XCHAIN_PROPOSALS, xc_proposal_id, chain_id), &report);

        env.storage()
            .persistent()
            .set(&(XCHAIN_PROPOSALS, xc_proposal_id), &xc_proposal);
    }

    /// Finalize a cross-chain proposal after all chains have reported
    pub fn finalize_cross_chain_proposal(env: &Env, xc_proposal_id: u64) -> bool {
        let mut xc_proposal: CrossChainProposal = env
            .storage()
            .persistent()
            .get(&(XCHAIN_PROPOSALS, xc_proposal_id))
            .expect("ERR_XC_PROPOSAL_NOT_FOUND: Cross-chain proposal not found");

        assert!(
            xc_proposal.quorum_met,
            "ERR_XC_QUORUM_NOT_MET: Not all chains have reported"
        );

        assert!(
            !xc_proposal.finalized,
            "ERR_XC_ALREADY_FINALIZED: Already finalized"
        );

        xc_proposal.finalized = true;

        let passed = xc_proposal.total_for_votes > xc_proposal.total_against_votes;

        env.storage()
            .persistent()
            .set(&(XCHAIN_PROPOSALS, xc_proposal_id), &xc_proposal);

        passed
    }

    // ========== View Functions ==========

    /// Get a cross-chain proposal by ID
    pub fn get_cross_chain_proposal(env: &Env, xc_proposal_id: u64) -> Option<CrossChainProposal> {
        env.storage()
            .persistent()
            .get(&(XCHAIN_PROPOSALS, xc_proposal_id))
    }

    /// Get a registered chain's info
    pub fn get_chain_info(env: &Env, chain_id: Bytes) -> Option<ChainInfo> {
        env.storage().persistent().get(&(CHAIN_REGISTRY, chain_id))
    }

    /// Get the cross-chain proposal count
    pub fn get_cross_chain_proposal_count(env: &Env) -> u64 {
        env.storage().instance().get(&XCHAIN_COUNT).unwrap_or(0)
    }
}
