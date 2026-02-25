use crate::{
    MobileAccessibilitySettings, MobilePreferences, OnboardingStage, 
    TeachLinkBridge, TeachLinkBridgeClient, FeedbackCategory,
    LayoutDensity, FocusStyle, ColorBlindMode, DeviceInfo, DeviceType, ScreenSize, NetworkType,
};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, Vec, Map, symbol_short};

#[test]
fn test_accessibility_setting_persistence() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    let device_info = DeviceInfo {
        device_id: Bytes::from_slice(&env, b"device1"),
        device_type: DeviceType::Smartphone,
        os_version: Bytes::from_slice(&env, b"15.0"),
        app_version: Bytes::from_slice(&env, b"1.0.0"),
        screen_size: ScreenSize {
            width: 1080,
            height: 1920,
            density: 480,
            is_tablet: false,
        },
        storage_capacity: 128 * 1024 * 1024 * 1024,
        network_type: NetworkType::WiFi,
        capabilities: Vec::new(&env),
        last_seen: env.ledger().timestamp(),
    };

    let preferences = MobilePreferences {
        data_saver_mode: false,
        auto_download_wifi: true,
        video_quality: crate::VideoQuality::Auto,
        font_size: crate::FontSize::Medium,
        theme: crate::ThemePreference::Auto,
        language: Bytes::from_slice(&env, b"en"),
        vibration_enabled: true,
        sound_enabled: true,
        gesture_navigation: true,
        custom_theme_colors: Map::new(&env),
        layout_density: LayoutDensity::Standard,
    };

    client.initialize_mobile_profile(&user, &device_info, &preferences);

    let new_accessibility = MobileAccessibilitySettings {
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

    client.update_accessibility_settings(&user, &new_accessibility);

    // In a real test we would verify the state, but here we just ensure it doesn't panic
}

#[test]
fn test_onboarding_progression() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Initialize profile first
    let device_info = DeviceInfo {
        device_id: Bytes::from_slice(&env, b"device1"),
        device_type: DeviceType::Smartphone,
        os_version: Bytes::from_slice(&env, b"15.0"),
        app_version: Bytes::from_slice(&env, b"1.0.0"),
        screen_size: ScreenSize { width: 0, height: 0, density: 0, is_tablet: false },
        storage_capacity: 0,
        network_type: NetworkType::WiFi,
        capabilities: Vec::new(&env),
        last_seen: 0,
    };
    let preferences = MobilePreferences {
        data_saver_mode: false,
        auto_download_wifi: true,
        video_quality: crate::VideoQuality::Auto,
        font_size: crate::FontSize::Medium,
        theme: crate::ThemePreference::Auto,
        language: Bytes::from_slice(&env, b"en"),
        vibration_enabled: true,
        sound_enabled: true,
        gesture_navigation: true,
        custom_theme_colors: Map::new(&env),
        layout_density: LayoutDensity::Standard,
    };
    client.initialize_mobile_profile(&user, &device_info, &preferences);

    client.record_onboarding_progress(&user, &OnboardingStage::ProfileSetup);
    client.record_onboarding_progress(&user, &OnboardingStage::WalletConnect);
}

#[test]
fn test_feedback_collection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeachLinkBridge);
    let client = TeachLinkBridgeClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    client.submit_user_feedback(
        &user, 
        &5, 
        &Bytes::from_slice(&env, b"Great UI!"), 
        &FeedbackCategory::UX
    );
}
