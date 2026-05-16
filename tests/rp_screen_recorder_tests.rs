use replaykit::{CameraPosition, ScreenRecorder};

#[test]
fn camera_position_raw_values_are_stable() {
    assert_eq!(CameraPosition::Front.as_raw(), 1);
    assert_eq!(CameraPosition::Back.as_raw(), 2);
}

#[test]
fn shared_recorder_exposes_consistent_state() {
    let recorder =
        ScreenRecorder::shared().expect("RPScreenRecorder.shared() should exist on macOS");
    let state = recorder
        .state()
        .expect("screen recorder state should be readable");
    assert_eq!(state.is_available, recorder.is_available());
    assert_eq!(state.is_recording, recorder.is_recording());
    assert_eq!(
        state.is_microphone_enabled,
        recorder.is_microphone_enabled()
    );
    assert_eq!(state.is_camera_enabled, recorder.is_camera_enabled());
}

#[test]
#[ignore = "requires user consent and ReplayKit capture entitlements"]
fn start_recording_requires_user_interaction() {
    let recorder =
        ScreenRecorder::shared().expect("RPScreenRecorder.shared() should exist on macOS");
    let _ = recorder.start_recording();
}
