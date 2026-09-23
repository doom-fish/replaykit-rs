use replaykit::{ReplayKitError, SystemBroadcastPickerView};

#[test]
fn system_broadcast_picker_reports_macos_unavailable() {
    assert!(!SystemBroadcastPickerView::is_supported_on_current_platform());
    let reason = SystemBroadcastPickerView::unsupported_reason();
    assert!(reason.contains("macOS"));
    let Err(error) = SystemBroadcastPickerView::new(None, true);
    assert_eq!(error, ReplayKitError::NotSupported(reason));
}
