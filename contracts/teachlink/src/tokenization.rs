use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::events::{ContentMintedEvent, MetadataUpdatedEvent, OwnershipTransferredEvent};
use crate::storage::{CONTENT_TOKENS, OWNERSHIP, OWNER_TOKENS, TOKEN_COUNTER};
use crate::types::{ContentMetadata, ContentToken, ContentType, TransferType};

pub struct ContentTokenization;

impl ContentTokenization {
    /// Get the next token ID and increment the counter
    fn get_next_token_id(env: &Env) -> u64 {
        let counter: u64 = env
            .storage()
            .persistent()
            .get(&TOKEN_COUNTER)
            .unwrap_or(0u64);
        let next_id = counter + 1;
        env.storage().persistent().set(&TOKEN_COUNTER, &next_id);
        next_id
    }

    /// Mint a new educational content token
    pub fn mint(
        env: &Env,
        creator: Address,
        title: Bytes,
        description: Bytes,
        content_type: ContentType,
        content_hash: Bytes,
        license_type: Bytes,
        tags: Vec<Bytes>,
        is_transferable: bool,
        royalty_percentage: u32,
    ) -> u64 {
        let timestamp = env.ledger().timestamp();
        let token_id = Self::get_next_token_id(env);

        let metadata = ContentMetadata {
            title: title.clone(),
            description: description.clone(),
            content_type: content_type.clone(),
            creator: creator.clone(),
            content_hash: content_hash.clone(),
            license_type: license_type.clone(),
            tags: tags.clone(),
            created_at: timestamp,
            updated_at: timestamp,
        };

        let token = ContentToken {
            token_id,
            metadata: metadata.clone(),
            owner: creator.clone(),
            minted_at: timestamp,
            is_transferable,
            royalty_percentage,
        };

        // Store the token
        env.storage()
            .persistent()
            .set(&(CONTENT_TOKENS, token_id), &token);

        // Store ownership mapping
        env.storage()
            .persistent()
            .set(&(OWNERSHIP, token_id), &creator);

        // Add token to owner's token list
        let mut owner_tokens: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(OWNER_TOKENS, creator.clone()))
            .unwrap_or(Vec::new(&env));
        owner_tokens.push_back(token_id);
        env.storage()
            .persistent()
            .set(&(OWNER_TOKENS, creator.clone()), &owner_tokens);

        // Emit event
        ContentMintedEvent {
            token_id,
            creator: creator.clone(),
            metadata,
        }
        .publish(env);

        token_id
    }

    /// Transfer ownership of a content token
    pub fn transfer(env: &Env, from: Address, to: Address, token_id: u64, notes: Option<Bytes>) {
        // Get the token
        let token: ContentToken = env
            .storage()
            .persistent()
            .get(&(CONTENT_TOKENS, token_id))
            .expect("Token does not exist");

        // Verify ownership
        if token.owner != from {
            panic!("Caller is not the owner");
        }

        // Check if transferable
        if !token.is_transferable {
            panic!("Token is not transferable");
        }

        // Update ownership
        env.storage().persistent().set(&(OWNERSHIP, token_id), &to);

        // Update token owner
        let mut updated_token = token.clone();
        updated_token.owner = to.clone();
        updated_token.metadata.updated_at = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&(CONTENT_TOKENS, token_id), &updated_token);

        // Remove from old owner's list
        let from_tokens: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(OWNER_TOKENS, from.clone()))
            .unwrap_or(Vec::new(env));
        let mut new_from_tokens = Vec::new(env);
        for i in 0..from_tokens.len() {
            let id = from_tokens.get(i).unwrap();
            if id != token_id {
                new_from_tokens.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&(OWNER_TOKENS, from.clone()), &new_from_tokens);

        // Add to new owner's list
        let mut to_tokens: Vec<u64> = env
            .storage()
            .persistent()
            .get(&(OWNER_TOKENS, to.clone()))
            .unwrap_or(Vec::new(env));
        to_tokens.push_back(token_id);
        env.storage()
            .persistent()
            .set(&(OWNER_TOKENS, to.clone()), &to_tokens);

        // Emit event
        OwnershipTransferredEvent {
            token_id,
            from: from.clone(),
            to: to.clone(),
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);

        // Record provenance (handled by provenance module)
        crate::provenance::ProvenanceTracker::record_transfer(
            env,
            token_id,
            Some(from.clone()),
            to,
            TransferType::Transfer,
            notes,
        );
    }

    /// Get a content token by ID
    pub fn get_token(env: &Env, token_id: u64) -> Option<ContentToken> {
        env.storage().persistent().get(&(CONTENT_TOKENS, token_id))
    }

    /// Get the owner of a token
    pub fn get_owner(env: &Env, token_id: u64) -> Option<Address> {
        env.storage().persistent().get(&(OWNERSHIP, token_id))
    }

    /// Check if an address owns a token
    pub fn is_owner(env: &Env, token_id: u64, address: Address) -> bool {
        Self::get_owner(env, token_id)
            .map(|owner| owner == address)
            .unwrap_or(false)
    }

    /// Get all tokens owned by an address
    pub fn get_owner_tokens(env: &Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&(OWNER_TOKENS, owner))
            .unwrap_or(Vec::new(env))
    }

    /// Get the total number of tokens minted
    pub fn get_token_count(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&TOKEN_COUNTER)
            .unwrap_or(0u64)
    }

    /// Update token metadata (only by owner)
    pub fn update_metadata(
        env: &Env,
        owner: Address,
        token_id: u64,
        title: Option<Bytes>,
        description: Option<Bytes>,
        tags: Option<Vec<Bytes>>,
    ) {
        let mut token: ContentToken = env
            .storage()
            .persistent()
            .get(&(CONTENT_TOKENS, token_id))
            .expect("Token does not exist");

        if token.owner != owner {
            panic!("Only owner can update metadata");
        }

        if let Some(new_title) = title {
            token.metadata.title = new_title;
        }

        if let Some(new_description) = description {
            token.metadata.description = new_description;
        }

        if let Some(new_tags) = tags {
            token.metadata.tags = new_tags;
        }

        token.metadata.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&(CONTENT_TOKENS, token_id), &token);

        // Emit event
        MetadataUpdatedEvent {
            token_id,
            owner: owner.clone(),
            timestamp: env.ledger().timestamp(),
        }
        .publish(env);
    }

    /// Set transferability of a token (only by owner)
    pub fn set_transferable(env: &Env, owner: Address, token_id: u64, transferable: bool) {
        let mut token: ContentToken = env
            .storage()
            .persistent()
            .get(&(CONTENT_TOKENS, token_id))
            .expect("Token does not exist");

        if token.owner != owner {
            panic!("Only owner can set transferability");
        }

        token.is_transferable = transferable;
        token.metadata.updated_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&(CONTENT_TOKENS, token_id), &token);
    }
}
