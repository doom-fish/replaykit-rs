use replaykit::{ReplayKitError, SystemBroadcastPickerView};

#[test]
fn system_broadcast_picker_reports_macos_unavailable() {
    assert!(!SystemBroadcastPickerView::is_supported_on_current_platform());
    let reason = SystemBroadcastPickerView::unsupported_reason();
    assert!(reason.contains("macOS"));
    match SystemBroadcastPickerView::new(None, true) {
        Err(ReplayKitError::NotSupported(message)) => assert!(message.contains("macOS")),
        other => panic!("expected not-supported error, got {other:?}"),
    }
}
