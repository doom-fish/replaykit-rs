use replaykit::{CaptureEvent, CaptureSample, SampleBufferCaptureSession, SampleBufferType};

#[test]
fn sample_buffer_type_raw_values_are_stable() {
    assert_eq!(SampleBufferType::Video.as_raw(), 1);
    assert_eq!(SampleBufferType::AudioApp.as_raw(), 2);
    assert_eq!(SampleBufferType::AudioMic.as_raw(), 3);
}

#[test]
fn capture_session_support_is_reported() {
    assert!(SampleBufferCaptureSession::is_supported_on_current_platform());
}

#[test]
fn capture_events_hold_sample_metadata() {
    let sample = CaptureSample {
        sample_type: SampleBufferType::Video,
        num_samples: 1,
        data_is_ready: true,
        presentation_time_seconds: Some(1.25),
        duration_seconds: Some(0.5),
        video_orientation: Some(1),
    };
    let event = CaptureEvent::Sample(sample.clone());
    match event {
        CaptureEvent::Sample(inner) => assert_eq!(inner, sample),
        CaptureEvent::Error(_) => panic!("expected sample event"),
    }
}
