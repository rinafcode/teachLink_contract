use soroban_sdk::{Address, Bytes, Env, Map, String, Vec};
use crate::types::*;
use crate::storage::*;
use crate::errors::CDNError;
use crate::events::*;

pub struct CDNManager;

impl CDNManager {
    /// Initialize the CDN system
    pub fn initialize(
        env: &Env,
        admin: Address,
        primary_region: String,
        max_nodes: u32,
    ) -> Result<(), CDNError> {
        // Check if already initialized
        if env.storage().instance().has(&CDN_CONFIG) {
            return Err(CDNError::AlreadyInitialized);
        }

        admin.require_auth();

        // Create CDN configuration
        let config = CDNConfig {
            admin: admin.clone(),
            primary_region: primary_region.clone(),
            max_nodes,
            initialized: true,
            total_nodes: 0,
            total_content: 0,
        };

        // Store configuration
        env.storage().instance().set(&CDN_CONFIG, &config);
        env.storage().instance().set(&CDN_ADMIN, &admin);
        env.storage().instance().set(&NODE_COUNT, &0u32);
        env.storage().instance().set(&CONTENT_COUNT, &0u64);

        // Initialize empty collections
        let empty_nodes: Map<String, CDNNode> = Map::new(env);
        let empty_content: Map<String, ContentItem> = Map::new(env);
        let empty_active_nodes: Vec<String> = Vec::new(env);

        env.storage().instance().set(&CDN_NODES, &empty_nodes);
        env.storage().instance().set(&CONTENT_ITEMS, &empty_content);
        env.storage().instance().set(&ACTIVE_NODES, &empty_active_nodes);

        // Emit initialization event
        env.events().publish((
            String::from_str(env, "cdn_initialized"),
            CDNInitializedEvent {
                admin,
                primary_region,
                max_nodes,
                timestamp: env.ledger().timestamp(),
            }
        ), ());

        Ok(())
    }

    /// Register a new CDN node
    pub fn register_node(
        env: &Env,
        admin: Address,
        node_id: String,
        region: String,
        endpoint: String,
        node_type: CDNNodeType,
        capacity: u64,
    ) -> Result<(), CDNError> {
        // Verify admin authorization
        let stored_admin: Address = env.storage().instance().get(&CDN_ADMIN)
            .ok_or(CDNError::NotInitialized)?;
        if admin != stored_admin {
            return Err(CDNError::Unauthorized);
        }
        admin.require_auth();

        // Check if node already exists
        let mut nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        if nodes.contains_key(node_id.clone()) {
            return Err(CDNError::NodeAlreadyExists);
        }

        // Check max nodes limit
        let config: CDNConfig = env.storage().instance().get(&CDN_CONFIG)
            .ok_or(CDNError::NotInitialized)?;
        
        let current_count: u32 = env.storage().instance().get(&NODE_COUNT).unwrap_or(0);
        if current_count >= config.max_nodes {
            return Err(CDNError::MaxNodesReached);
        }

        // Create new node
        let node = CDNNode {
            node_id: node_id.clone(),
            region: region.clone(),
            endpoint,
            node_type: node_type.clone(),
            capacity,
            current_load: 0,
            health_score: 100,
            last_health_check: env.ledger().timestamp(),
            is_active: true,
            bandwidth_limit: capacity * 10, // 10x capacity as bandwidth limit
            storage_used: 0,
        };

        // Store node
        nodes.set(node_id.clone(), node);
        env.storage().instance().set(&CDN_NODES, &nodes);

        // Update active nodes list
        let mut active_nodes: Vec<String> = env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env));
        active_nodes.push_back(node_id.clone());
        env.storage().instance().set(&ACTIVE_NODES, &active_nodes);

        // Update counters
        env.storage().instance().set(&NODE_COUNT, &(current_count + 1));

        // Update config
        let mut updated_config = config;
        updated_config.total_nodes = current_count + 1;
        env.storage().instance().set(&CDN_CONFIG, &updated_config);

        // Emit node registered event
        env.events().publish((
            String::from_str(env, "node_registered"),
            NodeRegisteredEvent {
                node_id,
                region,
                node_type,
                capacity,
                timestamp: env.ledger().timestamp(),
            }
        ), ());

        Ok(())
    }

    /// Update node health metrics
    pub fn update_node_health(
        env: &Env,
        node_id: String,
        health_score: u32,
        current_load: u64,
    ) -> Result<(), CDNError> {
        let mut nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        let mut node = nodes.get(node_id.clone())
            .ok_or(CDNError::NodeNotFound)?;

        // Update health metrics
        node.health_score = health_score.min(100);
        node.current_load = current_load;
        node.last_health_check = env.ledger().timestamp();

        // Deactivate node if health is critical
        if health_score < 20 {
            node.is_active = false;
        }

        nodes.set(node_id, node);
        env.storage().instance().set(&CDN_NODES, &nodes);

        Ok(())
    }

    /// Deactivate a CDN node
    pub fn deactivate_node(
        env: &Env,
        admin: Address,
        node_id: String,
    ) -> Result<(), CDNError> {
        let stored_admin: Address = env.storage().instance().get(&CDN_ADMIN)
            .ok_or(CDNError::NotInitialized)?;
        if admin != stored_admin {
            return Err(CDNError::Unauthorized);
        }
        admin.require_auth();

        let mut nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        let mut node = nodes.get(node_id.clone())
            .ok_or(CDNError::NodeNotFound)?;

        node.is_active = false;
        nodes.set(node_id.clone(), node);
        env.storage().instance().set(&CDN_NODES, &nodes);

        // Remove from active nodes list
        let mut active_nodes: Vec<String> = env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env));

        let mut new_active_nodes = Vec::new(env);
        for i in 0..active_nodes.len() {
            let active_node_id = active_nodes.get(i).unwrap();
            if active_node_id != node_id {
                new_active_nodes.push_back(active_node_id);
            }
        }
        env.storage().instance().set(&ACTIVE_NODES, &new_active_nodes);

        // Emit deactivation event
        env.events().publish((
            String::from_str(env, "node_deactivated"),
            NodeDeactivatedEvent {
                node_id,
                reason: String::from_str(env, "admin_deactivation"),
                timestamp: env.ledger().timestamp(),
            }
        ), ());

        Ok(())
    }

    /// Upload content to the CDN
    pub fn upload_content(
        env: &Env,
        uploader: Address,
        content_id: String,
        content_hash: Bytes,
        content_type: ContentType,
        size: u64,
        metadata: Map<String, String>,
    ) -> Result<(), CDNError> {
        uploader.require_auth();

        // Check if content already exists
        let mut content_items: Map<String, ContentItem> = env.storage().instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        if content_items.contains_key(content_id.clone()) {
            return Err(CDNError::ContentAlreadyExists);
        }

        // Validate content size (max 1GB for now)
        if size > 1_000_000_000 {
            return Err(CDNError::ContentTooLarge);
        }

        // Get active nodes for replication
        let active_nodes: Vec<String> = env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env));

        if active_nodes.is_empty() {
            return Err(CDNError::NoAvailableNodes);
        }

        // Select nodes for replication (up to 3 nodes)
        let mut replicas = Vec::new(env);
        let replica_count = active_nodes.len().min(3);
        for i in 0..replica_count {
            replicas.push_back(active_nodes.get(i).unwrap());
        }

        // Determine default cache policy based on content type
        let cache_policy = match content_type {
            ContentType::Video | ContentType::Audio => CachePolicy::LongTerm,
            ContentType::Image => CachePolicy::MediumTerm,
            ContentType::Document => CachePolicy::ShortTerm,
            ContentType::Interactive => CachePolicy::NoCache,
            ContentType::Archive => CachePolicy::Permanent,
        };

        // Determine default compression based on content type
        let compression = match content_type {
            ContentType::Video => CompressionType::H264,
            ContentType::Image => CompressionType::WebP,
            ContentType::Document => CompressionType::Gzip,
            _ => CompressionType::None,
        };

        // Create content item
        let content_item = ContentItem {
            content_id: content_id.clone(),
            content_hash,
            content_type: content_type.clone(),
            size,
            uploader: uploader.clone(),
            upload_timestamp: env.ledger().timestamp(),
            metadata,
            cache_policy,
            compression,
            replicas: replicas.clone(),
            access_count: 0,
            last_accessed: 0,
            is_encrypted: false,
            drm_enabled: false,
        };

        // Store content item
        content_items.set(content_id.clone(), content_item);
        env.storage().instance().set(&CONTENT_ITEMS, &content_items);

        // Update content counter
        let current_count: u64 = env.storage().instance().get(&CONTENT_COUNT).unwrap_or(0);
        env.storage().instance().set(&CONTENT_COUNT, &(current_count + 1));

        // Update config
        let mut config: CDNConfig = env.storage().instance().get(&CDN_CONFIG)
            .ok_or(CDNError::NotInitialized)?;
        config.total_content = current_count + 1;
        env.storage().instance().set(&CDN_CONFIG, &config);

        // Emit content uploaded event
        env.events().publish((
            String::from_str(env, "content_uploaded"),
            ContentUploadedEvent {
                content_id,
                uploader,
                content_type,
                size,
                replicas: replicas.len(),
                timestamp: env.ledger().timestamp(),
            }
        ), ());

        Ok(())
    }

    /// Get optimal delivery endpoint for content
    pub fn get_delivery_endpoint(
        env: &Env,
        content_id: String,
        user_location: Option<String>,
        quality: Option<StreamingQuality>,
    ) -> Result<DeliveryEndpoint, CDNError> {
        // Get content item
        let content_items: Map<String, ContentItem> = env.storage().instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let mut content_item = content_items.get(content_id.clone())
            .ok_or(CDNError::ContentNotFound)?;

        // Get nodes
        let nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        // Find best node for delivery
        let mut best_node: Option<CDNNode> = None;
        let mut best_score = 0u32;

        for i in 0..content_item.replicas.len() {
            let replica_node_id = content_item.replicas.get(i).unwrap();
            if let Some(node) = nodes.get(replica_node_id.clone()) {
                if !node.is_active {
                    continue;
                }

                let mut score = node.health_score;

                // Prefer nodes in the same region as user
                if let Some(ref user_loc) = user_location {
                    if node.region == *user_loc {
                        score += 50;
                    }
                }

                // Prefer nodes with lower load
                let load_penalty = ((node.current_load * 100) / node.capacity).min(50) as u32;
                score = score.saturating_sub(load_penalty);

                if score > best_score {
                    best_score = score;
                    best_node = Some(node);
                }
            }
        }

        let selected_node = best_node.ok_or(CDNError::NoAvailableNodes)?;

        // Calculate estimated latency based on region and load
        let base_latency = if user_location.is_some() && 
            user_location.as_ref().unwrap() == &selected_node.region {
            20 // Same region
        } else {
            100 // Different region
        };

        let load_latency = (selected_node.current_load * 50) / selected_node.capacity;
        let estimated_latency = base_latency + load_latency;

        // Determine cache status (simplified)
        let cache_status = if content_item.access_count > 0 {
            CacheStatus::Hit
        } else {
            CacheStatus::Miss
        };

        // Generate streaming manifest for video content
        let streaming_manifest = if content_item.content_type == ContentType::Video {
            // Create basic streaming manifest
            let profiles = streaming::StreamingManager::create_default_profiles(env, ContentType::Video);
            Some(StreamingManifest {
                manifest_url: String::from_str(env, "https://cdn.example.com/playlist.m3u8"),
                protocol: StreamingProtocol::HLS,
                profiles,
                segment_urls: Vec::new(env),
                duration: 3600, // 1 hour default
                is_live: false,
            })
        } else {
            None
        };

        // Update access count
        content_item.access_count += 1;
        content_item.last_accessed = env.ledger().timestamp();
        let mut updated_content_items = content_items;
        updated_content_items.set(content_id, content_item);
        env.storage().instance().set(&CONTENT_ITEMS, &updated_content_items);

        // Create delivery endpoint
        let endpoint = DeliveryEndpoint {
            url: String::from_str(env, "https://cdn.example.com/content"),
            node_id: selected_node.node_id,
            region: selected_node.region,
            estimated_latency,
            cache_status,
            streaming_manifest,
            security_token: None,
        };

        Ok(endpoint)
    }

    // ========== View Functions ==========

    /// Get CDN configuration
    pub fn get_config(env: &Env) -> Result<CDNConfig, CDNError> {
        env.storage().instance().get(&CDN_CONFIG)
            .ok_or(CDNError::NotInitialized)
    }

    /// Get node information
    pub fn get_node(env: &Env, node_id: String) -> Result<CDNNode, CDNError> {
        let nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        nodes.get(node_id).ok_or(CDNError::NodeNotFound)
    }

    /// Get content information
    pub fn get_content(env: &Env, content_id: String) -> Result<ContentItem, CDNError> {
        let content_items: Map<String, ContentItem> = env.storage().instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        content_items.get(content_id).ok_or(CDNError::ContentNotFound)
    }

    /// List all active nodes
    pub fn list_active_nodes(env: &Env) -> Result<Vec<String>, CDNError> {
        Ok(env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env)))
    }

    /// Get admin address
    pub fn get_admin(env: &Env) -> Result<Address, CDNError> {
        env.storage().instance().get(&CDN_ADMIN)
            .ok_or(CDNError::NotInitialized)
    }
}