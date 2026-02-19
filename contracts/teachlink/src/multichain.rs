//! Multi-Chain Support Module
//!
//! This module extends bridge functionality to support multiple blockchain networks
//! with configurable chain parameters and asset mappings.

use crate::errors::BridgeError;
use crate::events::{AssetRegisteredEvent, ChainAddedEvent, ChainUpdatedEvent};
use crate::storage::{ASSET_COUNTER, CHAIN_CONFIGS, MULTI_CHAIN_ASSETS, SUPPORTED_CHAINS};
use crate::types::{ChainAssetInfo, ChainConfig, MultiChainAsset};
use soroban_sdk::{Address, Bytes, Env, Map, Vec};

/// Maximum number of supported chains
pub const MAX_SUPPORTED_CHAINS: u32 = 100;

/// Maximum number of multi-chain assets
pub const MAX_MULTI_CHAIN_ASSETS: u32 = 1000;

/// Multi-Chain Manager
pub struct MultiChainManager;

impl MultiChainManager {
    /// Add a new supported chain
    pub fn add_chain(
        env: &Env,
        chain_id: u32,
        chain_name: Bytes,
        bridge_contract_address: Bytes,
        confirmation_blocks: u32,
        gas_price: u64,
    ) -> Result<(), BridgeError> {
        // Check if chain already exists
        let chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));

        let chain_count = chains.len();
        if chain_count >= MAX_SUPPORTED_CHAINS {
            return Err(BridgeError::InvalidChainConfiguration);
        }

        // Validate inputs
        if chain_id == 0 {
            return Err(BridgeError::InvalidChainConfiguration);
        }
        if chain_name.is_empty() || chain_name.len() > 64 {
            return Err(BridgeError::InvalidChainConfiguration);
        }
        if bridge_contract_address.is_empty() || bridge_contract_address.len() > 64 {
            return Err(BridgeError::InvalidChainConfiguration);
        }

        // Create chain config
        let chain_config = ChainConfig {
            chain_id,
            chain_name: chain_name.clone(),
            is_active: true,
            bridge_contract_address,
            confirmation_blocks,
            gas_price,
            last_updated: env.ledger().timestamp(),
        };

        // Store chain config
        let mut chain_configs: Map<u32, ChainConfig> = env
            .storage()
            .instance()
            .get(&CHAIN_CONFIGS)
            .unwrap_or_else(|| Map::new(env));
        chain_configs.set(chain_id, chain_config);
        env.storage().instance().set(&CHAIN_CONFIGS, &chain_configs);

        // Mark chain as supported
        let mut chains = chains;
        chains.set(chain_id, true);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        // Emit event
        ChainAddedEvent {
            chain_id,
            chain_name,
            added_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Update chain configuration
    pub fn update_chain(
        env: &Env,
        chain_id: u32,
        is_active: bool,
        confirmation_blocks: Option<u32>,
        gas_price: Option<u64>,
    ) -> Result<(), BridgeError> {
        // Get chain config
        let mut chain_configs: Map<u32, ChainConfig> = env
            .storage()
            .instance()
            .get(&CHAIN_CONFIGS)
            .unwrap_or_else(|| Map::new(env));

        let mut chain_config = chain_configs
            .get(chain_id)
            .ok_or(BridgeError::DestinationChainNotSupported)?;

        // Update fields
        chain_config.is_active = is_active;
        if let Some(blocks) = confirmation_blocks {
            chain_config.confirmation_blocks = blocks;
        }
        if let Some(price) = gas_price {
            chain_config.gas_price = price;
        }
        chain_config.last_updated = env.ledger().timestamp();

        // Store updated config
        chain_configs.set(chain_id, chain_config);
        env.storage().instance().set(&CHAIN_CONFIGS, &chain_configs);

        // Update supported chains
        let mut chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));
        chains.set(chain_id, is_active);
        env.storage().instance().set(&SUPPORTED_CHAINS, &chains);

        // Emit event
        ChainUpdatedEvent {
            chain_id,
            is_active,
            updated_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Register a multi-chain asset
    pub fn register_asset(
        env: &Env,
        asset_id: Bytes,
        stellar_token: Address,
        chain_configs: Map<u32, ChainAssetInfo>,
    ) -> Result<u64, BridgeError> {
        // Validate asset ID
        if asset_id.is_empty() || asset_id.len() > 64 {
            return Err(BridgeError::InvalidInput);
        }

        // Check asset limit
        let asset_counter: u64 = env.storage().instance().get(&ASSET_COUNTER).unwrap_or(0u64);
        if asset_counter >= MAX_MULTI_CHAIN_ASSETS as u64 {
            return Err(BridgeError::InvalidInput);
        }

        // Validate chain configs
        for (chain_id, config) in chain_configs.iter() {
            // Check if chain is supported
            let chains: Map<u32, bool> = env
                .storage()
                .instance()
                .get(&SUPPORTED_CHAINS)
                .unwrap_or_else(|| Map::new(env));
            if !chains.get(chain_id).unwrap_or(false) {
                return Err(BridgeError::DestinationChainNotSupported);
            }

            // Validate config
            if config.token_address.is_empty() || config.token_address.len() > 64 {
                return Err(BridgeError::InvalidInput);
            }
        }

        let new_asset_counter = asset_counter + 1;

        // Create multi-chain asset
        let asset = MultiChainAsset {
            asset_id: asset_id.clone(),
            stellar_token: stellar_token.clone(),
            chain_configs: chain_configs.clone(),
            total_bridged: 0,
            is_active: true,
        };

        // Store asset
        let mut assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));
        assets.set(new_asset_counter, asset);
        env.storage().instance().set(&MULTI_CHAIN_ASSETS, &assets);
        env.storage()
            .instance()
            .set(&ASSET_COUNTER, &new_asset_counter);

        // Emit event
        AssetRegisteredEvent {
            asset_id,
            stellar_token,
            supported_chains: chain_configs.len(),
        }
        .publish(env);

        Ok(new_asset_counter)
    }

    /// Update asset status
    pub fn update_asset_status(
        env: &Env,
        asset_id: u64,
        is_active: bool,
    ) -> Result<(), BridgeError> {
        let mut assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));

        let mut asset = assets.get(asset_id).ok_or(BridgeError::AssetNotSupported)?;

        asset.is_active = is_active;
        assets.set(asset_id, asset);
        env.storage().instance().set(&MULTI_CHAIN_ASSETS, &assets);

        Ok(())
    }

    /// Update bridged amount for an asset
    pub fn update_bridged_amount(
        env: &Env,
        asset_id: u64,
        amount: i128,
        is_outgoing: bool,
    ) -> Result<(), BridgeError> {
        let mut assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));

        let mut asset = assets.get(asset_id).ok_or(BridgeError::AssetNotSupported)?;

        if is_outgoing {
            asset.total_bridged += amount;
        } else {
            asset.total_bridged -= amount;
        }

        assets.set(asset_id, asset);
        env.storage().instance().set(&MULTI_CHAIN_ASSETS, &assets);

        Ok(())
    }

    /// Check if a chain is supported and active
    pub fn is_chain_active(env: &Env, chain_id: u32) -> bool {
        let chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));

        if !chains.get(chain_id).unwrap_or(false) {
            return false;
        }

        // Check chain config
        let chain_configs: Map<u32, ChainConfig> = env
            .storage()
            .instance()
            .get(&CHAIN_CONFIGS)
            .unwrap_or_else(|| Map::new(env));

        if let Some(config) = chain_configs.get(chain_id) {
            config.is_active
        } else {
            false
        }
    }

    /// Get chain configuration
    pub fn get_chain_config(env: &Env, chain_id: u32) -> Option<ChainConfig> {
        let chain_configs: Map<u32, ChainConfig> = env
            .storage()
            .instance()
            .get(&CHAIN_CONFIGS)
            .unwrap_or_else(|| Map::new(env));
        chain_configs.get(chain_id)
    }

    /// Get multi-chain asset
    pub fn get_asset(env: &Env, asset_id: u64) -> Option<MultiChainAsset> {
        let assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));
        assets.get(asset_id)
    }

    /// Get chain asset info for a specific chain
    pub fn get_chain_asset_info(env: &Env, asset_id: u64, chain_id: u32) -> Option<ChainAssetInfo> {
        if let Some(asset) = Self::get_asset(env, asset_id) {
            asset.chain_configs.get(chain_id)
        } else {
            None
        }
    }

    /// Get all supported chains
    pub fn get_supported_chains(env: &Env) -> Vec<u32> {
        let chains: Map<u32, bool> = env
            .storage()
            .instance()
            .get(&SUPPORTED_CHAINS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (chain_id, is_supported) in chains.iter() {
            if is_supported {
                result.push_back(chain_id);
            }
        }
        result
    }

    /// Get all active assets
    pub fn get_active_assets(env: &Env) -> Vec<u64> {
        let assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (asset_id, asset) in assets.iter() {
            if asset.is_active {
                result.push_back(asset_id);
            }
        }
        result
    }

    /// Validate cross-chain transfer
    pub fn validate_cross_chain_transfer(
        env: &Env,
        source_chain: u32,
        destination_chain: u32,
        asset_id: u64,
    ) -> Result<(), BridgeError> {
        // Check if source chain is active
        if !Self::is_chain_active(env, source_chain) {
            return Err(BridgeError::ChainNotActive);
        }

        // Check if destination chain is active
        if !Self::is_chain_active(env, destination_chain) {
            return Err(BridgeError::DestinationChainNotSupported);
        }

        // Check if asset is supported
        let assets: Map<u64, MultiChainAsset> = env
            .storage()
            .instance()
            .get(&MULTI_CHAIN_ASSETS)
            .unwrap_or_else(|| Map::new(env));

        let asset = assets.get(asset_id).ok_or(BridgeError::AssetNotSupported)?;

        if !asset.is_active {
            return Err(BridgeError::AssetNotSupported);
        }

        // Check if asset is configured for both chains
        if !asset.chain_configs.contains_key(source_chain) {
            return Err(BridgeError::AssetNotSupported);
        }
        if !asset.chain_configs.contains_key(destination_chain) {
            return Err(BridgeError::AssetNotSupported);
        }

        Ok(())
    }
}
