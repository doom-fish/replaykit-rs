use replaykit::{
    BroadcastActivityControllerHandle, BroadcastActivityViewController, ReplayKitError,
};

#[test]
fn ios_activity_view_controller_reports_macos_unavailable() {
    assert!(!BroadcastActivityViewController::is_supported_on_current_platform());
    let reason = BroadcastActivityViewController::unsupported_reason();
    assert!(reason.contains("macOS"));
    match BroadcastActivityViewController::load() {
        Err(ReplayKitError::NotSupported(message)) => assert!(message.contains("macOS")),
        other => panic!("expected not-supported error, got {other:?}"),
    }
}

#[test]
fn macos_activity_controller_handle_reports_supported() {
    assert!(BroadcastActivityControllerHandle::is_supported_on_current_platform());
}
