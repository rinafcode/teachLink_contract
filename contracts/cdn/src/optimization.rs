use crate::errors::CDNError;
use crate::events::*;
use crate::storage::*;
use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Vec};

pub struct OptimizationManager;

impl OptimizationManager {
    /// Update cache policy for content
    pub fn update_cache_policy(
        env: &Env,
        admin: Address,
        content_id: String,
        cache_policy: CachePolicy,
    ) -> Result<(), CDNError> {
        // Verify admin authorization
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&CDN_ADMIN)
            .ok_or(CDNError::NotInitialized)?;
        if admin != stored_admin {
            return Err(CDNError::Unauthorized);
        }
        admin.require_auth();

        // Get content item
        let mut content_items: Map<String, ContentItem> = env
            .storage()
            .instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let mut content_item = content_items
            .get(content_id.clone())
            .ok_or(CDNError::ContentNotFound)?;

        let old_policy = content_item.cache_policy.clone();
        content_item.cache_policy = cache_policy.clone();

        content_items.set(content_id.clone(), content_item);
        env.storage().instance().set(&CONTENT_ITEMS, &content_items);

        // Emit cache policy updated event
        env.events().publish(
            (
                String::from_str(env, "cache_policy_updated"),
                CachePolicyUpdatedEvent {
                    content_id,
                    old_policy,
                    new_policy: cache_policy,
                    timestamp: env.ledger().timestamp(),
                },
            ),
            (),
        );

        Ok(())
    }

    /// Optimize content compression
    pub fn optimize_compression(
        env: &Env,
        admin: Address,
        content_id: String,
        compression_type: CompressionType,
    ) -> Result<(), CDNError> {
        // Verify admin authorization
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&CDN_ADMIN)
            .ok_or(CDNError::NotInitialized)?;
        if admin != stored_admin {
            return Err(CDNError::Unauthorized);
        }
        admin.require_auth();

        // Get content item
        let mut content_items: Map<String, ContentItem> = env
            .storage()
            .instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let mut content_item = content_items
            .get(content_id.clone())
            .ok_or(CDNError::ContentNotFound)?;

        // Validate compression type for content type
        let is_valid = Self::validate_compression_for_content_type(
            &content_item.content_type,
            &compression_type,
        );

        if !is_valid {
            return Err(CDNError::UnsupportedCompression);
        }

        let old_size = content_item.size;
        let old_compression = content_item.compression.clone();

        // Update compression (in real implementation, this would trigger actual compression)
        content_item.compression = compression_type;

        // Estimate new size based on compression type (simplified)
        let new_size = Self::estimate_compressed_size(old_size, &content_item.compression);
        content_item.size = new_size;

        content_items.set(content_id.clone(), content_item);
        env.storage().instance().set(&CONTENT_ITEMS, &content_items);

        // Calculate savings
        let savings_percentage = if old_size > 0 {
            ((old_size - new_size) * 100) / old_size
        } else {
            0
        };

        // Emit optimization applied event
        env.events().publish(
            (
                String::from_str(env, "optimization_applied"),
                OptimizationAppliedEvent {
                    content_id,
                    optimization_type: OptimizationType::Compression,
                    old_size,
                    new_size,
                    savings_percentage: savings_percentage as u32,
                    timestamp: env.ledger().timestamp(),
                },
            ),
            (),
        );

        Ok(())
    }

    /// Get optimization recommendations for content
    pub fn get_recommendations(
        env: &Env,
        content_id: String,
    ) -> Result<Vec<OptimizationRecommendation>, CDNError> {
        let content_items: Map<String, ContentItem> = env
            .storage()
            .instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let content_item = content_items
            .get(content_id)
            .ok_or(CDNError::ContentNotFound)?;

        let mut recommendations = Vec::new(env);

        // Compression recommendations
        Self::add_compression_recommendations(env, &content_item, &mut recommendations);

        // Cache policy recommendations
        Self::add_cache_recommendations(env, &content_item, &mut recommendations);

        // Replication recommendations
        Self::add_replication_recommendations(env, &content_item, &mut recommendations);

        Ok(recommendations)
    }

    /// Calculate cost optimization for content delivery
    pub fn calculate_cost_optimization(
        env: &Env,
        content_id: String,
        target_regions: Vec<String>,
    ) -> Result<CostOptimization, CDNError> {
        let content_items: Map<String, ContentItem> = env
            .storage()
            .instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let content_item = content_items
            .get(content_id)
            .ok_or(CDNError::ContentNotFound)?;

        // Get content analytics for cost calculation
        let analytics_map: Map<String, ContentAnalytics> = env
            .storage()
            .instance()
            .get(&CONTENT_ANALYTICS)
            .unwrap_or_else(|| Map::new(env));

        let analytics = analytics_map
            .get(content_item.content_id.clone())
            .unwrap_or_else(|| ContentAnalytics {
                content_id: content_item.content_id.clone(),
                total_requests: 0,
                total_bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                top_regions: Vec::new(env),
                bandwidth_usage: 0,
            });

        // Calculate current cost (simplified model)
        let current_cost = Self::calculate_current_cost(&content_item, &analytics);

        // Calculate optimized cost with target regions
        let optimized_cost =
            Self::calculate_optimized_cost(env, &content_item, &analytics, &target_regions);

        let savings = if current_cost > optimized_cost {
            current_cost - optimized_cost
        } else {
            0
        };

        // Generate cost optimization recommendations
        let mut recommendations = Vec::new(env);

        if savings > 0 {
            recommendations.push_back(String::from_str(env, "Optimize regional distribution"));
        }

        if content_item.compression == CompressionType::None {
            recommendations.push_back(String::from_str(
                env,
                "Enable compression to reduce bandwidth costs",
            ));
        }

        if analytics.cache_hit_ratio < 70 {
            recommendations.push_back(String::from_str(
                env,
                "Improve cache policy to reduce origin requests",
            ));
        }

        Ok(CostOptimization {
            current_cost,
            optimized_cost,
            savings,
            recommendations,
        })
    }

    // ========== Helper Functions ==========

    /// Validate if compression type is suitable for content type
    fn validate_compression_for_content_type(
        content_type: &ContentType,
        compression_type: &CompressionType,
    ) -> bool {
        match (content_type, compression_type) {
            (
                ContentType::Video,
                CompressionType::H264 | CompressionType::H265 | CompressionType::AV1,
            ) => true,
            (ContentType::Image, CompressionType::WebP | CompressionType::AVIF) => true,
            (
                ContentType::Document | ContentType::Interactive,
                CompressionType::Gzip | CompressionType::Brotli,
            ) => true,
            (ContentType::Archive, CompressionType::Gzip) => true,
            (_, CompressionType::None) => true,
            _ => false,
        }
    }

    /// Estimate compressed size based on compression type
    fn estimate_compressed_size(original_size: u64, compression_type: &CompressionType) -> u64 {
        match compression_type {
            CompressionType::None => original_size,
            CompressionType::Gzip => (original_size * 70) / 100, // ~30% reduction
            CompressionType::Brotli => (original_size * 65) / 100, // ~35% reduction
            CompressionType::WebP => (original_size * 75) / 100, // ~25% reduction
            CompressionType::AVIF => (original_size * 60) / 100, // ~40% reduction
            CompressionType::H264 => (original_size * 80) / 100, // ~20% reduction
            CompressionType::H265 => (original_size * 60) / 100, // ~40% reduction
            CompressionType::AV1 => (original_size * 50) / 100,  // ~50% reduction
        }
    }

    /// Add compression recommendations
    fn add_compression_recommendations(
        env: &Env,
        content_item: &ContentItem,
        recommendations: &mut Vec<OptimizationRecommendation>,
    ) {
        if content_item.compression == CompressionType::None && content_item.size > 1_000_000 {
            let recommended_compression = match content_item.content_type {
                ContentType::Video => "H265 or AV1 for better compression",
                ContentType::Image => "WebP or AVIF for smaller file sizes",
                ContentType::Document => "Gzip or Brotli for text compression",
                _ => "Enable appropriate compression",
            };

            recommendations.push_back(OptimizationRecommendation {
                recommendation_type: OptimizationType::Compression,
                description: String::from_str(env, recommended_compression),
                estimated_savings: (content_item.size * 30) / 100, // Estimate 30% savings
                priority: 8,
            });
        }
    }

    /// Add cache policy recommendations
    fn add_cache_recommendations(
        env: &Env,
        content_item: &ContentItem,
        recommendations: &mut Vec<OptimizationRecommendation>,
    ) {
        if content_item.access_count > 100 && content_item.cache_policy == CachePolicy::NoCache {
            recommendations.push_back(OptimizationRecommendation {
                recommendation_type: OptimizationType::Caching,
                description: String::from_str(
                    env,
                    "Enable caching for frequently accessed content",
                ),
                estimated_savings: content_item.access_count * 50, // Estimate latency savings
                priority: 9,
            });
        }

        if content_item.access_count > 1000 && content_item.cache_policy == CachePolicy::ShortTerm {
            recommendations.push_back(OptimizationRecommendation {
                recommendation_type: OptimizationType::Caching,
                description: String::from_str(
                    env,
                    "Consider longer cache policy for popular content",
                ),
                estimated_savings: content_item.access_count * 20,
                priority: 7,
            });
        }
    }

    /// Add replication recommendations
    fn add_replication_recommendations(
        env: &Env,
        content_item: &ContentItem,
        recommendations: &mut Vec<OptimizationRecommendation>,
    ) {
        if content_item.replicas.len() < 3 && content_item.access_count > 500 {
            recommendations.push_back(OptimizationRecommendation {
                recommendation_type: OptimizationType::Replication,
                description: String::from_str(env, "Add more replicas for better availability"),
                estimated_savings: content_item.access_count * 10, // Estimate performance improvement
                priority: 6,
            });
        }
    }

    /// Calculate current delivery cost
    fn calculate_current_cost(content_item: &ContentItem, analytics: &ContentAnalytics) -> u64 {
        // Simplified cost model: base cost + bandwidth cost + storage cost
        let base_cost = 100; // Base cost per content item
        let bandwidth_cost = (analytics.bandwidth_usage * 5) / 1_000_000; // $5 per GB
        let storage_cost = (content_item.size * 2) / 1_000_000; // $2 per GB storage
        let replica_cost = content_item.replicas.len() as u64 * storage_cost;

        base_cost + bandwidth_cost + storage_cost + replica_cost
    }

    /// Calculate optimized delivery cost
    fn calculate_optimized_cost(
        env: &Env,
        content_item: &ContentItem,
        analytics: &ContentAnalytics,
        target_regions: &Vec<String>,
    ) -> u64 {
        // Calculate cost with optimized regional distribution
        let base_cost = 100;

        // Reduced bandwidth cost due to better regional distribution
        let optimized_bandwidth_cost = (analytics.bandwidth_usage * 3) / 1_000_000; // $3 per GB (reduced)

        // Storage cost based on target regions
        let storage_cost = (content_item.size * 2) / 1_000_000;
        let optimized_replica_cost = target_regions.len() as u64 * storage_cost;

        // Compression savings
        let compression_savings = if content_item.compression == CompressionType::None {
            storage_cost * 30 / 100 // 30% savings with compression
        } else {
            0
        };

        let total_cost =
            base_cost + optimized_bandwidth_cost + storage_cost + optimized_replica_cost;

        if total_cost > compression_savings {
            total_cost - compression_savings
        } else {
            0
        }
    }
}
