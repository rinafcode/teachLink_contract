use crate::errors::EscrowError;
use crate::storage::{ARBITRATORS, ESCROWS};
use crate::types::{ArbitratorProfile, Escrow, EscrowStatus};
use soroban_sdk::{Address, Env, Map, String, Vec};

pub struct ArbitrationManager;

impl ArbitrationManager {
    /// Register a new professional arbitrator
    pub fn register_arbitrator(env: &Env, profile: ArbitratorProfile) -> Result<(), EscrowError> {
        profile.address.require_auth();

        let mut arbitrators: Map<Address, ArbitratorProfile> = env
            .storage()
            .instance()
            .get(&ARBITRATORS)
            .unwrap_or_else(|| Map::new(env));

        arbitrators.set(profile.address.clone(), profile);
        env.storage().instance().set(&ARBITRATORS, &arbitrators);

        Ok(())
    }

    /// Update arbitrator profile
    pub fn update_profile(
        env: &Env,
        address: Address,
        profile: ArbitratorProfile,
    ) -> Result<(), EscrowError> {
        address.require_auth();
        if address != profile.address {
            return Err(EscrowError::SignerNotAuthorized);
        }

        let mut arbitrators: Map<Address, ArbitratorProfile> = env
            .storage()
            .instance()
            .get(&ARBITRATORS)
            .unwrap_or_else(|| Map::new(env));

        if !arbitrators.contains_key(address.clone()) {
            return Err(EscrowError::SignerNotAuthorized);
        }

        arbitrators.set(address, profile);
        env.storage().instance().set(&ARBITRATORS, &arbitrators);
        Ok(())
    }

    /// Get arbitrator profile
    pub fn get_arbitrator(env: &Env, address: Address) -> Option<ArbitratorProfile> {
        let arbitrators: Map<Address, ArbitratorProfile> = env
            .storage()
            .instance()
            .get(&ARBITRATORS)
            .unwrap_or_else(|| Map::new(env));
        arbitrators.get(address)
    }

    /// Pick an active arbitrator for an escrow dispute
    pub fn pick_arbitrator(env: &Env) -> Result<Address, EscrowError> {
        let arbitrators: Map<Address, ArbitratorProfile> = env
            .storage()
            .instance()
            .get(&ARBITRATORS)
            .unwrap_or_else(|| Map::new(env));

        for (addr, profile) in arbitrators.iter() {
            if profile.is_active {
                return Ok(addr);
            }
        }

        Err(EscrowError::ArbitratorNotAuthorized) // No active arbitrators found
    }

    /// Automated dispute detection: check if an escrow has stalled
    pub fn check_stalled_escrow(env: &Env, escrow: &Escrow) -> bool {
        if escrow.status != EscrowStatus::Pending {
            return false;
        }

        let now = env.ledger().timestamp();
        let timeout = 604800; // 7 days in seconds

        // If it's been pending too long since creation without approvals
        if now > escrow.created_at + timeout && escrow.approval_count == 0 {
            return true;
        }

        false
    }

    /// Update reputation after a resolution
    pub fn update_reputation(
        env: &Env,
        arbitrator_addr: Address,
        success: bool,
    ) -> Result<(), EscrowError> {
        let mut arbitrators: Map<Address, ArbitratorProfile> = env
            .storage()
            .instance()
            .get(&ARBITRATORS)
            .unwrap_or_else(|| Map::new(env));

        let mut profile = match arbitrators.get(arbitrator_addr.clone()) {
            Some(p) => p,
            None => return Ok(()), // Not a registered professional arbitrator
        };

        profile.total_resolved += 1;
        if success {
            profile.reputation_score = profile.reputation_score.saturating_add(10).min(1000);
        } else {
            profile.reputation_score = profile.reputation_score.saturating_sub(20);
        }

        arbitrators.set(arbitrator_addr, profile);
        env.storage().instance().set(&ARBITRATORS, &arbitrators);
        Ok(())
    }
}
