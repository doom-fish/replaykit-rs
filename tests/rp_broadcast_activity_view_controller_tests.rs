use replaykit::{
    BroadcastActivityControllerHandle, BroadcastActivityViewController, ReplayKitError,
};

#[test]
fn ios_activity_view_controller_reports_macos_unavailable() {
    assert!(!BroadcastActivityViewController::is_supported_on_current_platform());
    let reason = BroadcastActivityViewController::unsupported_reason();
    assert!(reason.contains("macOS"));
    let Err(error) = BroadcastActivityViewController::load();
    assert_eq!(error, ReplayKitError::NotSupported(reason));
}

#[test]
fn macos_activity_controller_handle_reports_supported() {
    assert!(BroadcastActivityControllerHandle::is_supported_on_current_platform());
}
