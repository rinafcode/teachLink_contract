//! Unit tests for Mobile Platform types and logic.
//! These tests validate the type construction, field assignments, and enum variants
//! used for the mobile UI/UX enhancements, without requiring a deployed contract client.

use soroban_sdk::{Bytes, Env, Map};
use teachlink_contract::{
    ColorBlindMode, DeviceInfo, FeedbackCategory, FocusStyle, FontSize, LayoutDensity,
    MobileAccessibilitySettings, MobilePreferences, OnboardingStage, ThemePreference, VideoQuality,
};

#[test]
fn test_accessibility_type_construction() {
    let env = Env::default();

    let accessibility = MobileAccessibilitySettings {
        screen_reader_enabled: true,
        high_contrast_enabled: true,
        large_text_enabled: true,
        voice_control_enabled: false,
        gesture_navigation_enabled: true,
        haptic_feedback_enabled: true,
        color_blind_mode: ColorBlindMode::Deuteranopia,
        reduced_motion_enabled: true,
        focus_indicator_style: FocusStyle::HighVisibility,
    };

    assert!(accessibility.screen_reader_enabled);
    assert!(accessibility.high_contrast_enabled);
    assert!(!accessibility.voice_control_enabled);
}

#[test]
fn test_device_info_type_construction() {
    let env = Env::default();

    let device_info = DeviceInfo {
        device_id: Bytes::from_slice(&env, b"device1"),
        model: Bytes::from_slice(&env, b"iPhone15"),
        os_version: Bytes::from_slice(&env, b"15.0"),
        push_token: Bytes::from_slice(&env, b"tok"),
        screen_resolution: Bytes::from_slice(&env, b"1080x1920"),
        is_tablet: false,
    };

    assert!(!device_info.is_tablet);
}

#[test]
fn test_mobile_preferences_type_construction() {
    let env = Env::default();

    let preferences = MobilePreferences {
        data_saver_mode: false,
        auto_download_wifi: true,
        video_quality: VideoQuality::High,
        font_size: FontSize::Large,
        theme: ThemePreference::Dark,
        language: Bytes::from_slice(&env, b"en"),
        vibration_enabled: true,
        sound_enabled: true,
        gesture_navigation: true,
        custom_theme_colors: Map::new(&env),
        layout_density: LayoutDensity::Comfortable,
    };

    assert!(preferences.auto_download_wifi);
    assert!(!preferences.data_saver_mode);
}

#[test]
fn test_onboarding_stages() {
    // Verify all expected onboarding stages exist as variants
    let _stages: Vec<OnboardingStage> = vec![
        OnboardingStage::ProfileSetup,
        OnboardingStage::WalletConnect,
        OnboardingStage::FirstCourse,
        OnboardingStage::CommunityJoin,
        OnboardingStage::SecuritySetup,
    ];
}

#[test]
fn test_feedback_category_variants() {
    // Verify all expected feedback categories exist
    let _cats: Vec<FeedbackCategory> = vec![
        FeedbackCategory::UX,
        FeedbackCategory::Performance,
        FeedbackCategory::Content,
        FeedbackCategory::Bug,
        FeedbackCategory::FeatureRequest,
    ];
}
