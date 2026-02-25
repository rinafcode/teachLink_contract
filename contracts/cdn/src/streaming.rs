use crate::errors::CDNError;

use crate::storage::*;
use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Vec};

pub struct StreamingManager;

#[allow(deprecated)]
impl StreamingManager {
    /// Create adaptive streaming configuration for content
    pub fn create_adaptive_config(
        env: &Env,
        admin: Address,
        content_id: String,
        protocol: StreamingProtocol,
        profiles: Vec<StreamingProfile>,
        segment_duration: u32,
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

        // Verify content exists and is video type
        let content_items: Map<String, ContentItem> = env
            .storage()
            .instance()
            .get(&CONTENT_ITEMS)
            .unwrap_or_else(|| Map::new(env));

        let content_item = content_items
            .get(content_id.clone())
            .ok_or(CDNError::ContentNotFound)?;

        if content_item.content_type != ContentType::Video {
            return Err(CDNError::InvalidContentType);
        }

        // Validate streaming profiles
        if profiles.is_empty() {
            return Err(CDNError::InvalidInput);
        }

        // Create adaptive streaming configuration
        let adaptive_config = AdaptiveStreamingConfig {
            protocol: protocol.clone(),
            profiles: profiles.clone(),
            segment_duration,
            playlist_type: String::from_str(env, "VOD"),
            encryption_enabled: content_item.is_encrypted,
            drm_enabled: content_item.drm_enabled,
        };

        // Store streaming configuration
        let mut streaming_configs: Map<String, AdaptiveStreamingConfig> = env
            .storage()
            .instance()
            .get(&STREAMING_CONFIGS)
            .unwrap_or_else(|| Map::new(env));

        streaming_configs.set(content_id.clone(), adaptive_config);
        env.storage()
            .instance()
            .set(&STREAMING_CONFIGS, &streaming_configs);

        // Emit streaming configuration event
        env.events().publish(
            (
                String::from_str(env, "streaming_config_created"),
                content_id,
                protocol,
                profiles.len(),
                env.ledger().timestamp(),
            ),
            (),
        );

        Ok(())
    }

    /// Generate streaming manifest based on network conditions
    pub fn generate_manifest(
        env: &Env,
        content_id: String,
        network_condition: NetworkCondition,
        user_preferences: Option<StreamingQuality>,
    ) -> Result<StreamingManifest, CDNError> {
        // Get streaming configuration
        let streaming_configs: Map<String, AdaptiveStreamingConfig> = env
            .storage()
            .instance()
            .get(&STREAMING_CONFIGS)
            .unwrap_or_else(|| Map::new(env));

        let config = streaming_configs
            .get(content_id.clone())
            .ok_or(CDNError::ContentNotFound)?;

        // Select optimal profiles based on network conditions
        let optimal_profiles = Self::select_optimal_profiles(
            env,
            &config.profiles,
            &network_condition,
            user_preferences,
        );

        // Generate segment URLs
        let segment_urls = Self::generate_segment_urls(env, &content_id, &optimal_profiles);

        // Create manifest
        let manifest = StreamingManifest {
            manifest_url: String::from_str(env, "https://cdn.example.com/manifest.m3u8"),
            protocol: config.protocol,
            profiles: optimal_profiles,
            segment_urls,
            duration: 3600, // 1 hour default
            is_live: false,
        };

        // Cache manifest for future requests
        let mut manifest_cache: Map<String, StreamingManifest> = env
            .storage()
            .instance()
            .get(&MANIFEST_CACHE)
            .unwrap_or_else(|| Map::new(env));

        manifest_cache.set(content_id, manifest.clone());
        env.storage()
            .instance()
            .set(&MANIFEST_CACHE, &manifest_cache);

        Ok(manifest)
    }

    /// Adapt streaming quality based on real-time network conditions
    pub fn adapt_streaming_quality(
        env: &Env,
        content_id: String,
        current_quality: StreamingQuality,
        network_condition: NetworkCondition,
    ) -> Result<StreamingQuality, CDNError> {
        // Get streaming configuration
        let streaming_configs: Map<String, AdaptiveStreamingConfig> = env
            .storage()
            .instance()
            .get(&STREAMING_CONFIGS)
            .unwrap_or_else(|| Map::new(env));

        let config = streaming_configs
            .get(content_id)
            .ok_or(CDNError::ContentNotFound)?;

        // Determine optimal quality based on network conditions
        let optimal_quality = Self::calculate_optimal_quality(&network_condition, &config.profiles);

        // Apply adaptive logic - avoid frequent quality changes
        let adapted_quality =
            Self::apply_adaptive_logic(env, current_quality, optimal_quality, &network_condition);

        Ok(adapted_quality)
    }

    /// Monitor network conditions and suggest quality adjustments
    pub fn monitor_network_conditions(
        env: &Env,
        user: Address,
        content_id: String,
        network_metrics: NetworkCondition,
    ) -> Result<Vec<String>, CDNError> {
        let mut recommendations = Vec::new(env);

        // Analyze network conditions and provide recommendations
        if network_metrics.bandwidth < 1_000_000 {
            // Less than 1 Mbps
            recommendations.push_back(String::from_str(
                env,
                "Switch to low quality for better playback",
            ));
        } else if network_metrics.bandwidth < 5_000_000 {
            // Less than 5 Mbps
            recommendations.push_back(String::from_str(env, "Medium quality recommended"));
        } else if network_metrics.bandwidth > 25_000_000 {
            // More than 25 Mbps
            recommendations.push_back(String::from_str(env, "High or Ultra quality available"));
        }

        if network_metrics.latency > 200 {
            // High latency
            recommendations.push_back(String::from_str(
                env,
                "High latency detected - consider lower quality",
            ));
        }

        if network_metrics.packet_loss > 5 {
            // High packet loss
            recommendations.push_back(String::from_str(
                env,
                "Network instability - adaptive streaming recommended",
            ));
        }

        if network_metrics.stability_score < 50 {
            recommendations.push_back(String::from_str(
                env,
                "Unstable connection - enable aggressive buffering",
            ));
        }

        Ok(recommendations)
    }

    /// Get streaming analytics for content
    pub fn get_streaming_analytics(
        env: &Env,
        content_id: String,
    ) -> Result<Map<String, u64>, CDNError> {
        let mut analytics = Map::new(env);

        // Get content analytics
        let content_analytics_map: Map<String, ContentAnalytics> = env
            .storage()
            .instance()
            .get(&CONTENT_ANALYTICS)
            .unwrap_or_else(|| Map::new(env));

        if let Some(content_analytics) = content_analytics_map.get(content_id) {
            analytics.set(
                String::from_str(env, "total_streams"),
                content_analytics.total_requests,
            );
            analytics.set(
                String::from_str(env, "total_bytes_streamed"),
                content_analytics.total_bytes_served,
            );
            analytics.set(
                String::from_str(env, "avg_streaming_time"),
                content_analytics.average_response_time,
            );
            analytics.set(
                String::from_str(env, "cache_hit_ratio"),
                content_analytics.cache_hit_ratio as u64,
            );
        }

        Ok(analytics)
    }

    // ========== Helper Functions ==========

    /// Select optimal streaming profiles based on network conditions
    fn select_optimal_profiles(
        env: &Env,
        available_profiles: &Vec<StreamingProfile>,
        network_condition: &NetworkCondition,
        user_preference: Option<StreamingQuality>,
    ) -> Vec<StreamingProfile> {
        let mut selected_profiles = Vec::new(env);

        // If user has a specific preference, try to honor it
        if let Some(preferred_quality) = user_preference {
            for i in 0..available_profiles.len() {
                let profile = available_profiles.get(i).unwrap();
                if profile.quality == preferred_quality {
                    selected_profiles.push_back(profile);
                    return selected_profiles;
                }
            }
        }

        // Select profiles based on network bandwidth
        for i in 0..available_profiles.len() {
            let profile = available_profiles.get(i).unwrap();
            let required_bandwidth = (profile.bitrate as u64) * 1000; // Convert kbps to bps

            // Include profile if network can handle it with some buffer
            if network_condition.bandwidth > required_bandwidth * 120 / 100 {
                // 20% buffer
                selected_profiles.push_back(profile);
            }
        }

        // Always include at least the lowest quality profile
        if selected_profiles.is_empty() && !available_profiles.is_empty() {
            selected_profiles.push_back(available_profiles.get(0).unwrap());
        }

        selected_profiles
    }

    /// Generate segment URLs for streaming profiles
    fn generate_segment_urls(
        env: &Env,
        content_id: &String,
        profiles: &Vec<StreamingProfile>,
    ) -> Vec<String> {
        let mut segment_urls = Vec::new(env);

        // Generate URLs for each quality profile
        for i in 0..profiles.len() {
            let profile = profiles.get(i).unwrap();
            let quality_str = match profile.quality {
                StreamingQuality::Low => "480p",
                StreamingQuality::Medium => "720p",
                StreamingQuality::High => "1080p",
                StreamingQuality::Ultra => "4k",
                StreamingQuality::Adaptive => "adaptive",
            };

            let segment_url = String::from_str(env, "https://cdn.example.com/segments/");
            segment_urls.push_back(segment_url);
        }

        segment_urls
    }

    /// Calculate optimal quality based on network conditions
    fn calculate_optimal_quality(
        network_condition: &NetworkCondition,
        profiles: &Vec<StreamingProfile>,
    ) -> StreamingQuality {
        let bandwidth_mbps = network_condition.bandwidth / 1_000_000;

        // Quality selection based on available bandwidth
        if bandwidth_mbps >= 25 && network_condition.stability_score > 80 {
            StreamingQuality::Ultra
        } else if bandwidth_mbps >= 8 && network_condition.stability_score > 70 {
            StreamingQuality::High
        } else if bandwidth_mbps >= 3 && network_condition.stability_score > 60 {
            StreamingQuality::Medium
        } else {
            StreamingQuality::Low // Fallback to lowest quality for all other cases
        }
    }

    /// Apply adaptive logic to avoid frequent quality changes
    fn apply_adaptive_logic(
        env: &Env,
        current_quality: StreamingQuality,
        optimal_quality: StreamingQuality,
        network_condition: &NetworkCondition,
    ) -> StreamingQuality {
        // If network is stable, allow quality changes
        if network_condition.stability_score > 80 {
            return optimal_quality;
        }

        // If network is unstable, be conservative with quality changes
        if network_condition.stability_score < 50 {
            // Only downgrade quality if absolutely necessary
            match (current_quality, optimal_quality.clone()) {
                (StreamingQuality::Ultra, StreamingQuality::Low) => StreamingQuality::Medium,
                (StreamingQuality::High, StreamingQuality::Low) => StreamingQuality::Medium,
                _ => optimal_quality,
            }
        } else {
            optimal_quality
        }
    }

    /// Create default streaming profiles for content
    pub fn create_default_profiles(env: &Env, content_type: ContentType) -> Vec<StreamingProfile> {
        let mut profiles = Vec::new(env);

        match content_type {
            ContentType::Video => {
                // Low quality profile
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::Low,
                    bitrate: 800,
                    resolution_width: 854,
                    resolution_height: 480,
                    framerate: 30,
                    codec: String::from_str(env, "H264"),
                });

                // Medium quality profile
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::Medium,
                    bitrate: 2500,
                    resolution_width: 1280,
                    resolution_height: 720,
                    framerate: 30,
                    codec: String::from_str(env, "H264"),
                });

                // High quality profile
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::High,
                    bitrate: 5000,
                    resolution_width: 1920,
                    resolution_height: 1080,
                    framerate: 30,
                    codec: String::from_str(env, "H265"),
                });

                // Ultra quality profile
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::Ultra,
                    bitrate: 15000,
                    resolution_width: 3840,
                    resolution_height: 2160,
                    framerate: 60,
                    codec: String::from_str(env, "AV1"),
                });
            }
            ContentType::Audio => {
                // Audio profiles
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::Low,
                    bitrate: 64,
                    resolution_width: 0,
                    resolution_height: 0,
                    framerate: 0,
                    codec: String::from_str(env, "AAC"),
                });

                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::High,
                    bitrate: 320,
                    resolution_width: 0,
                    resolution_height: 0,
                    framerate: 0,
                    codec: String::from_str(env, "AAC"),
                });
            }
            _ => {
                // Default profile for other content types
                profiles.push_back(StreamingProfile {
                    quality: StreamingQuality::Medium,
                    bitrate: 1000,
                    resolution_width: 0,
                    resolution_height: 0,
                    framerate: 0,
                    codec: String::from_str(env, "Default"),
                });
            }
        }

        profiles
    }
}
