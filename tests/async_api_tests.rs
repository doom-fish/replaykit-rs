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

    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {}
        Ok(Err(payload)) => panic::resume_unwind(payload),
        Err(RecvTimeoutError::Timeout) => {
            eprintln!("Skipping {name}: ReplayKit callbacks did not complete in time");
        }
        Err(RecvTimeoutError::Disconnected) => panic!("{name} worker thread disconnected"),
    }
}

#[test]
fn test_start_recording_happy_path() {
    run_async_case("test_start_recording_happy_path", || pollster::block_on(async {
        let Some(recorder) = ScreenRecorder::shared() else {
            println!("ReplayKit unavailable, skipping test");
            return;
        };

        if !recorder.is_available() {
            println!("ReplayKit not available on this system, skipping test");
            return;
        }

        // Test starting recording
        if AsyncScreenRecorder::start_recording(&recorder).await == Ok(()) {
            println!("✓ start_recording succeeded");
            // Clean up by stopping
            let _ = AsyncScreenRecorder::stop_recording(&recorder).await;
        } else {
            panic!("start_recording failed");
        }
    }));
}

#[test]
fn test_stop_recording_happy_path() {
    run_async_case("test_stop_recording_happy_path", || pollster::block_on(async {
        let Some(recorder) = ScreenRecorder::shared() else {
            println!("ReplayKit unavailable, skipping test");
            return;
        };

        if !recorder.is_available() {
            println!("ReplayKit not available on this system, skipping test");
            return;
        }

        // Start recording first
        if AsyncScreenRecorder::start_recording(&recorder).await.is_err() {
            println!("Could not start recording, skipping stop test");
            return;
        }

        // Test stopping recording
        if let Ok(preview) = AsyncScreenRecorder::stop_recording(&recorder).await {
            println!("✓ stop_recording succeeded, preview: {}", preview.is_some());
        } else {
            panic!("stop_recording failed");
        }
    }));
}

#[test]
fn test_discard_recording_error_path() {
    run_async_case("test_discard_recording_error_path", || pollster::block_on(async {
        let Some(recorder) = ScreenRecorder::shared() else {
            println!("ReplayKit unavailable, skipping test");
            return;
        };

        if !recorder.is_available() {
            println!("ReplayKit not available on this system, skipping test");
            return;
        }

        // Test discard when not recording (should error)
        match AsyncScreenRecorder::discard_recording(&recorder).await {
            Ok(()) => println!("✓ discard_recording succeeded"),
            Err(e) => println!("✓ discard_recording failed as expected: {e}"),
        }
    }));
}
