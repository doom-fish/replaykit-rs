#![cfg(all(test, feature = "async"))]

use std::{
    panic::{self, AssertUnwindSafe},
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

use replaykit::async_api::AsyncScreenRecorder;
use replaykit::prelude::*;

fn run_async_case(name: &str, body: impl FnOnce() + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = panic::catch_unwind(AssertUnwindSafe(body));
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(90)) {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => panic::resume_unwind(payload),
        Err(RecvTimeoutError::Timeout) => {
            panic!("{name}: ReplayKit callbacks did not complete in time");
        }
        Err(RecvTimeoutError::Disconnected) => panic!("{name} worker thread disconnected"),
    }
}

#[test]
fn stopping_without_a_recording_fails_the_same_way_in_both_bridges() {
    run_async_case(
        "stopping_without_a_recording_fails_the_same_way_in_both_bridges",
        || {
            let recorder = ScreenRecorder::shared().expect("shared recorder");
            assert!(!recorder.is_recording());

            let sync_error = recorder
                .stop_recording()
                .expect_err("stopping without a recording must fail");
            let async_error = pollster::block_on(AsyncScreenRecorder::stop_recording(&recorder))
                .expect_err("stopping without a recording must fail");

            assert_eq!(async_error, sync_error);
            let ReplayKitError::Framework(error) = async_error else {
                panic!("expected a framework error, got {async_error:?}");
            };
            assert_eq!(
                error.recording_code(),
                Some(RecordingErrorCode::AttemptToStopNonRecording)
            );
        },
    );
}

#[test]
#[ignore = "starts a screen recording, which requires screen-recording consent"]
fn test_start_recording_happy_path() {
    run_async_case("test_start_recording_happy_path", || {
        pollster::block_on(async {
            let recorder = ScreenRecorder::shared().expect("shared recorder");
            assert!(recorder.is_available(), "ReplayKit is unavailable");

            assert_eq!(AsyncScreenRecorder::start_recording(&recorder).await, Ok(()));
            assert!(recorder.is_recording());
            assert!(AsyncScreenRecorder::stop_recording(&recorder).await.is_ok());
        });
    });
}

#[test]
#[ignore = "starts a screen recording, which requires screen-recording consent"]
fn test_stop_recording_happy_path() {
    run_async_case("test_stop_recording_happy_path", || {
        pollster::block_on(async {
            let recorder = ScreenRecorder::shared().expect("shared recorder");
            assert!(recorder.is_available(), "ReplayKit is unavailable");

            assert_eq!(AsyncScreenRecorder::start_recording(&recorder).await, Ok(()));
            let preview = AsyncScreenRecorder::stop_recording(&recorder)
                .await
                .expect("stop_recording failed");
            assert!(!recorder.is_recording());
            if let Some(preview) = preview {
                assert_eq!(preview.class_name(), "RPPreviewViewController");
            }
        });
    });
}

#[test]
#[ignore = "discarding needs a finished recording, which requires screen-recording consent"]
fn test_discard_recording_after_stop() {
    run_async_case("test_discard_recording_after_stop", || {
        pollster::block_on(async {
            let recorder = ScreenRecorder::shared().expect("shared recorder");
            assert!(recorder.is_available(), "ReplayKit is unavailable");

            assert_eq!(AsyncScreenRecorder::start_recording(&recorder).await, Ok(()));
            assert!(AsyncScreenRecorder::stop_recording(&recorder).await.is_ok());
            assert_eq!(AsyncScreenRecorder::discard_recording(&recorder).await, Ok(()));
        });
    });
}
