use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::events::ProvenanceRecordedEvent;
use crate::storage::PROVENANCE;
use crate::types::{ProvenanceRecord, TransferType};

pub struct ProvenanceTracker;

impl ProvenanceTracker {
    /// Record a transfer in the provenance chain
    pub fn record_transfer(
        env: &Env,
        token_id: u64,
        from: Option<Address>,
        to: Address,
        transfer_type: TransferType,
        notes: Option<Bytes>,
    ) {
        let timestamp = env.ledger().timestamp();
        
        // Get transaction hash (using ledger sequence as a proxy)
        let tx_hash = Bytes::from_slice(
            env,
            &env.ledger().sequence().to_be_bytes(),
        );

        let record = ProvenanceRecord {
            token_id,
            from,
            to: to.clone(),
            timestamp,
            transaction_hash: tx_hash,
            transfer_type: transfer_type.clone(),
            notes,
        };

        // Get existing provenance history
        let mut history: Vec<ProvenanceRecord> = env
            .storage()
            .persistent()
            .get(&(PROVENANCE, token_id))
            .unwrap_or(Vec::new(env));

        // Add new record
        history.push_back(record.clone());

        // Store updated history
        env.storage()
            .persistent()
            .set(&(PROVENANCE, token_id), &history);

        // Emit event
        ProvenanceRecordedEvent {
            token_id,
            record,
        }
        .publish(env);
    }

    /// Record initial mint in provenance
    pub fn record_mint(
        env: &Env,
        token_id: u64,
        creator: Address,
        notes: Option<Bytes>,
    ) {
        Self::record_transfer(
            env,
            token_id,
            None, // No previous owner for mint
            creator,
            TransferType::Mint,
            notes,
        );
    }

    /// Get full provenance history for a token
    pub fn get_provenance(env: &Env, token_id: u64) -> Vec<ProvenanceRecord> {
        env.storage()
            .persistent()
            .get(&(PROVENANCE, token_id))
            .unwrap_or(Vec::new(env))
    }

    /// Get the number of transfers for a token
    pub fn get_transfer_count(env: &Env, token_id: u64) -> u32 {
        let history = Self::get_provenance(env, token_id);
        history.len()
    }

    /// Verify ownership chain integrity
    pub fn verify_chain(env: &Env, token_id: u64) -> bool {
        let history = Self::get_provenance(env, token_id);
        
        if history.len() == 0 {
            return false;
        }

        // First record should be a mint
        let first = history.get(0).unwrap();
        if first.transfer_type != TransferType::Mint {
            return false;
        }

        // Verify chain continuity
        for i in 1..history.len() {
            let prev = history.get(i - 1).unwrap();
            let curr = history.get(i).unwrap();

            // Previous 'to' should match current 'from' (or None for mint)
            let prev_to = prev.to.clone();
            if prev_to != curr.from.unwrap_or(prev_to.clone()) {
                return false;
            }

            // Timestamps should be in order
            if curr.timestamp < prev.timestamp {
                return false;
            }
        }

        true
    }

    /// Get the original creator of a token
    pub fn get_creator(env: &Env, token_id: u64) -> Option<Address> {
        let history = Self::get_provenance(env, token_id);
        if history.len() == 0 {
            return None;
        }

        let first = history.get(0).unwrap();
        if first.transfer_type == TransferType::Mint {
            Some(first.to)
        } else {
            None
        }
    }

    /// Get all addresses that have owned this token
    pub fn get_all_owners(env: &Env, token_id: u64) -> Vec<Address> {
        let history = Self::get_provenance(env, token_id);
        let mut owners = Vec::new(env);

        for i in 0..history.len() {
            let record = history.get(i).unwrap();
            // Add the 'to' address (new owner)
            if !owners.contains(record.to.clone()) {
                owners.push_back(record.to);
            }
        }

        owners
    }
}
