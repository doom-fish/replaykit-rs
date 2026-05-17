/// Example: Async screen recording with `ReplayKit`
///
/// This example demonstrates how to use the async API to start and stop recording
/// without blocking the thread. This uses `pollster` to run async code in a sync context.
/// On macOS 11+, this will attempt to start recording, wait a moment, and then stop.
#[cfg(feature = "async")]
fn main() {
    use std::thread;
    use std::time::Duration;

    use replaykit::prelude::*;
    use replaykit::async_api::AsyncScreenRecorder;

    let Some(recorder) = ScreenRecorder::shared() else {
        println!("ReplayKit unavailable");
        return;
    };

    if !recorder.is_available() {
        println!("ReplayKit is not available on this system");
        return;
    }

    println!("Starting async recording...");
    pollster::block_on(async {
        match AsyncScreenRecorder::start_recording(&recorder).await {
            Ok(()) => println!("Recording started successfully"),
            Err(e) => {
                println!("Failed to start recording: {e}");
                return;
            }
        }

        // Record for a short time
        thread::sleep(Duration::from_secs(2));

        println!("Stopping recording...");
        if let Ok(preview) = AsyncScreenRecorder::stop_recording(&recorder).await {
            if preview.is_some() {
                println!("Recording stopped and preview available");
            } else {
                println!("Recording stopped");
            }
        } else {
            println!("Failed to stop recording");
        }
    });
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature. Run with: cargo run --example 01_async_recording --features async");
}
