use replaykit::{PreviewEvent, PreviewViewController};

#[test]
fn preview_controller_support_is_reported() {
    assert!(PreviewViewController::is_supported_on_current_platform());
}

#[test]
fn preview_events_hold_activity_types() {
    let event = PreviewEvent::DidFinishWithActivityTypes(vec!["com.apple.share".to_owned()]);
    match event {
        PreviewEvent::DidFinishWithActivityTypes(activity_types) => {
            assert_eq!(activity_types, vec!["com.apple.share".to_owned()]);
        }
        PreviewEvent::DidFinish => panic!("expected activity-types event"),
    }
}
