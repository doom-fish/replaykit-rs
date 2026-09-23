use replaykit::{BroadcastConfiguration, ReplayKitError};

#[test]
fn broadcast_configuration_reports_macos_unavailable() {
    assert!(!BroadcastConfiguration::is_supported_on_current_platform());
    let reason = BroadcastConfiguration::unsupported_reason();
    assert!(reason.contains("macOS") || reason.contains("deprecated"));
    let Err(error) = BroadcastConfiguration::new();
    assert_eq!(error, ReplayKitError::NotSupported(reason));
}
