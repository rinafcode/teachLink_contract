use soroban_sdk::{Env, Map, String, Vec};
use crate::types::*;
use crate::storage::*;
use crate::errors::CDNError;
use crate::events::*;

pub struct AnalyticsManager;

impl AnalyticsManager {
    /// Record content access for analytics
    pub fn record_access(
        env: &Env,
        content_id: String,
        user_location: String,
        node_id: String,
        bytes_served: u64,
        response_time: u64,
    ) -> Result<(), CDNError> {
        // Update global metrics
        Self::update_global_metrics(env, bytes_served, response_time)?;

        // Update content-specific analytics
        Self::update_content_analytics(env, content_id.clone(), bytes_served, response_time)?;

        // Update regional metrics
        Self::update_regional_metrics(env, user_location.clone(), bytes_served, response_time)?;

        // Determine cache status based on response time (simplified)
        let cache_status = if response_time < 100 {
            CacheStatus::Hit
        } else {
            CacheStatus::Miss
        };

        // Emit content accessed event
        env.events().publish((
            String::from_str(env, "content_accessed"),
            ContentAccessedEvent {
                content_id,
                node_id,
                user_location,
                bytes_served,
                response_time,
                cache_status,
                timestamp: env.ledger().timestamp(),
            }
        ), ());

        Ok(())
    }

    /// Get content analytics
    pub fn get_content_analytics(
        env: &Env,
        content_id: String,
        time_range: Option<TimeRange>,
    ) -> Result<ContentAnalytics, CDNError> {
        let analytics_map: Map<String, ContentAnalytics> = env.storage().instance()
            .get(&CONTENT_ANALYTICS)
            .unwrap_or_else(|| Map::new(env));

        if let Some(analytics) = analytics_map.get(content_id.clone()) {
            // If time range is specified, we would filter the data
            // For simplicity, we return the full analytics
            Ok(analytics)
        } else {
            // Return empty analytics if no data exists
            let empty_regions = Vec::new(env);
            Ok(ContentAnalytics {
                content_id,
                total_requests: 0,
                total_bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                top_regions: empty_regions,
                bandwidth_usage: 0,
            })
        }
    }

    /// Get global CDN metrics
    pub fn get_global_metrics(env: &Env) -> Result<GlobalMetrics, CDNError> {
        if let Some(metrics) = env.storage().instance().get(&GLOBAL_METRICS) {
            Ok(metrics)
        } else {
            // Return default metrics if none exist
            Ok(GlobalMetrics {
                total_requests: 0,
                total_bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                active_nodes: 0,
                total_content_items: 0,
                bandwidth_usage: 0,
            })
        }
    }

    /// Get regional performance metrics
    pub fn get_regional_metrics(
        env: &Env,
        region: String,
    ) -> Result<RegionalMetrics, CDNError> {
        let regional_metrics_map: Map<String, RegionalMetrics> = env.storage().instance()
            .get(&REGIONAL_METRICS)
            .unwrap_or_else(|| Map::new(env));

        if let Some(metrics) = regional_metrics_map.get(region.clone()) {
            Ok(metrics)
        } else {
            // Return empty metrics if no data exists for this region
            Ok(RegionalMetrics {
                region,
                requests: 0,
                bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                active_nodes: 0,
            })
        }
    }

    // ========== Internal Helper Functions ==========

    /// Update global metrics
    fn update_global_metrics(
        env: &Env,
        bytes_served: u64,
        response_time: u64,
    ) -> Result<(), CDNError> {
        let mut metrics: GlobalMetrics = env.storage().instance()
            .get(&GLOBAL_METRICS)
            .unwrap_or_else(|| GlobalMetrics {
                total_requests: 0,
                total_bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                active_nodes: 0,
                total_content_items: 0,
                bandwidth_usage: 0,
            });

        // Update metrics
        metrics.total_requests += 1;
        metrics.total_bytes_served += bytes_served;
        metrics.bandwidth_usage += bytes_served;

        // Calculate new average response time
        if metrics.total_requests > 1 {
            let total_response_time = (metrics.average_response_time * (metrics.total_requests - 1)) + response_time;
            metrics.average_response_time = total_response_time / metrics.total_requests;
        } else {
            metrics.average_response_time = response_time;
        }

        // Update cache hit ratio (simplified - based on response time)
        let cache_hits = if response_time < 100 { 1 } else { 0 };
        let total_cache_score = (metrics.cache_hit_ratio * (metrics.total_requests - 1)) + (cache_hits * 100);
        metrics.cache_hit_ratio = total_cache_score / metrics.total_requests;

        // Get current active nodes and content count
        let active_nodes: Vec<String> = env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env));
        metrics.active_nodes = active_nodes.len();

        let content_count: u64 = env.storage().instance().get(&CONTENT_COUNT).unwrap_or(0);
        metrics.total_content_items = content_count;

        env.storage().instance().set(&GLOBAL_METRICS, &metrics);

        Ok(())
    }

    /// Update content-specific analytics
    fn update_content_analytics(
        env: &Env,
        content_id: String,
        bytes_served: u64,
        response_time: u64,
    ) -> Result<(), CDNError> {
        let mut analytics_map: Map<String, ContentAnalytics> = env.storage().instance()
            .get(&CONTENT_ANALYTICS)
            .unwrap_or_else(|| Map::new(env));

        let mut analytics = analytics_map.get(content_id.clone())
            .unwrap_or_else(|| ContentAnalytics {
                content_id: content_id.clone(),
                total_requests: 0,
                total_bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                top_regions: Vec::new(env),
                bandwidth_usage: 0,
            });

        // Update analytics
        analytics.total_requests += 1;
        analytics.total_bytes_served += bytes_served;
        analytics.bandwidth_usage += bytes_served;

        // Calculate new average response time
        if analytics.total_requests > 1 {
            let total_response_time = (analytics.average_response_time * (analytics.total_requests - 1)) + response_time;
            analytics.average_response_time = total_response_time / analytics.total_requests;
        } else {
            analytics.average_response_time = response_time;
        }

        // Update cache hit ratio (simplified)
        let cache_hits = if response_time < 100 { 1 } else { 0 };
        let total_cache_score = (analytics.cache_hit_ratio * (analytics.total_requests - 1)) + (cache_hits * 100);
        analytics.cache_hit_ratio = total_cache_score / analytics.total_requests;

        analytics_map.set(content_id, analytics);
        env.storage().instance().set(&CONTENT_ANALYTICS, &analytics_map);

        Ok(())
    }

    /// Update regional metrics
    fn update_regional_metrics(
        env: &Env,
        region: String,
        bytes_served: u64,
        response_time: u64,
    ) -> Result<(), CDNError> {
        let mut regional_metrics_map: Map<String, RegionalMetrics> = env.storage().instance()
            .get(&REGIONAL_METRICS)
            .unwrap_or_else(|| Map::new(env));

        let mut metrics = regional_metrics_map.get(region.clone())
            .unwrap_or_else(|| RegionalMetrics {
                region: region.clone(),
                requests: 0,
                bytes_served: 0,
                average_response_time: 0,
                cache_hit_ratio: 0,
                active_nodes: 0,
            });

        // Update metrics
        metrics.requests += 1;
        metrics.bytes_served += bytes_served;

        // Calculate new average response time
        if metrics.requests > 1 {
            let total_response_time = (metrics.average_response_time * (metrics.requests - 1)) + response_time;
            metrics.average_response_time = total_response_time / metrics.requests;
        } else {
            metrics.average_response_time = response_time;
        }

        // Update cache hit ratio (simplified)
        let cache_hits = if response_time < 100 { 1 } else { 0 };
        let total_cache_score = (metrics.cache_hit_ratio * (metrics.requests - 1)) + (cache_hits * 100);
        metrics.cache_hit_ratio = total_cache_score / metrics.requests;

        // Count active nodes in this region
        let nodes: Map<String, CDNNode> = env.storage().instance()
            .get(&CDN_NODES)
            .unwrap_or_else(|| Map::new(env));

        let mut active_nodes_count = 0u32;
        let active_nodes: Vec<String> = env.storage().instance()
            .get(&ACTIVE_NODES)
            .unwrap_or_else(|| Vec::new(env));

        for i in 0..active_nodes.len() {
            let node_id = active_nodes.get(i).unwrap();
            if let Some(node) = nodes.get(node_id) {
                if node.region == region && node.is_active {
                    active_nodes_count += 1;
                }
            }
        }
        metrics.active_nodes = active_nodes_count;

        regional_metrics_map.set(region, metrics);
        env.storage().instance().set(&REGIONAL_METRICS, &regional_metrics_map);

        Ok(())
    }

    /// Generate performance alerts based on metrics
    pub fn check_performance_alerts(env: &Env) -> Result<Vec<String>, CDNError> {
        let mut alerts = Vec::new(env);

        // Check global metrics for alerts
        if let Some(global_metrics) = env.storage().instance().get(&GLOBAL_METRICS) {
            // Alert if average response time is too high
            if global_metrics.average_response_time > 1000 { // 1 second
                alerts.push_back(String::from_str(env, "High average response time detected"));
            }

            // Alert if cache hit ratio is too low
            if global_metrics.cache_hit_ratio < 50 { // Less than 50%
                alerts.push_back(String::from_str(env, "Low cache hit ratio detected"));
            }

            // Alert if no active nodes
            if global_metrics.active_nodes == 0 {
                alerts.push_back(String::from_str(env, "No active nodes available"));
            }
        }

        Ok(alerts)
    }
}