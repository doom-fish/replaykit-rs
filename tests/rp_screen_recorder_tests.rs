use replaykit::{CameraPosition, ReplayKitError, ScreenRecorder};

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
fn unknown_camera_positions_are_rejected() {
    let recorder =
        ScreenRecorder::shared().expect("RPScreenRecorder.shared() should exist on macOS");
    let before = recorder.camera_position();
    assert!(matches!(
        recorder.set_camera_position(CameraPosition::Unknown(99)),
        Err(ReplayKitError::InvalidArgument(_))
    ));
    assert_eq!(recorder.camera_position(), before);
}

#[test]
fn clip_exports_reject_invalid_durations() {
    let recorder =
        ScreenRecorder::shared().expect("RPScreenRecorder.shared() should exist on macOS");
    for duration in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            recorder.export_clip_to_output("target/never-written.mov", duration),
            Err(ReplayKitError::InvalidArgument(_))
        ));
    }
}

#[test]
#[ignore = "requires user consent and ReplayKit capture entitlements"]
fn start_recording_requires_user_interaction() {
    let recorder =
        ScreenRecorder::shared().expect("RPScreenRecorder.shared() should exist on macOS");
    assert_eq!(recorder.start_recording(), Ok(()));
    assert!(recorder.is_recording());
    assert_eq!(recorder.stop_recording(), Ok(()));
    assert!(!recorder.is_recording());
}
