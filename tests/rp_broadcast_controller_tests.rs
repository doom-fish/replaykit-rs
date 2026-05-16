use replaykit::{BroadcastActivityControllerHandle, BroadcastController, BroadcastControllerEvent};
use serde_json::json;

#[test]
fn broadcast_api_support_is_reported() {
    assert!(BroadcastController::is_supported_on_current_platform());
    assert!(BroadcastActivityControllerHandle::is_supported_on_current_platform());
}

#[test]
fn broadcast_events_are_constructible() {
    let event = BroadcastControllerEvent::DidUpdateServiceInfo(json!({"status": "ready"}));
    match event {
        BroadcastControllerEvent::DidUpdateServiceInfo(value) => {
            assert_eq!(value["status"], json!("ready"));
        }
        _ => panic!("expected service info event"),
    }
}

#[test]
#[ignore = "requires user interaction with the system broadcast picker"]
fn show_picker_can_be_invoked() {
    BroadcastActivityControllerHandle::show((0.0, 0.0), None, |_| {});
}
