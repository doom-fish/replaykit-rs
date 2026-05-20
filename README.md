# replaykit-rs

Safe Rust bindings for Apple's **`ReplayKit`** framework on macOS.

[![Crates.io](https://img.shields.io/crates/v/replaykit-rs.svg)](https://crates.io/crates/replaykit-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

## Covered areas

- `RPScreenRecorder` state, microphone/camera controls, camera preview access, recording, direct-to-file recording, clip buffering, and typed delegate callbacks
- `RPBroadcastController` start/pause/resume/finish, `serviceInfo`, and delegate events
- macOS `RPBroadcastActivityController` through `BroadcastActivityControllerHandle::show`
- `RPPreviewViewController` delegate callbacks and support helpers
- `RPSampleBufferType` and `RPScreenRecorder.startCapture` via `SampleBufferCaptureSession`
- Broadcast-extension authoring helpers via `BroadcastExtensionContext`, `BroadcastHandler`, `BroadcastSampleHandler`, and `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY`
- Explicit `NotSupported` wrappers for macOS-unavailable `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration`
- Typed `RPRecordingErrorCode` mapping plus replay/broadcast error domains
- **Async API**: executor-agnostic futures for recording + broadcast-picker flows, plus bounded async streams for broadcast-controller, preview-controller, detailed recorder, and sample-buffer capture events via the `async` feature

## Requirements

- macOS 11.0+
- Xcode with Swift toolchain installed

## Quick start

```rust
use replaykit::ScreenRecorder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recorder = ScreenRecorder::shared().expect("ReplayKit unavailable");
    let state = recorder.state()?;
    println!("ReplayKit available: {}", state.is_available);
    println!("Recording: {}", state.is_recording);
    Ok(())
}
```

## Async feature

Enable `async` to get:

- `AsyncScreenRecorder::{start_recording, stop_recording, stop_recording_with_output, discard_recording, detailed_events, capture_events}`
- `AsyncBroadcastActivityControllerHandle::show(...)`
- `BroadcastControllerEventStream`, `PreviewEventStream`, `DetailedRecordingEventStream`, and `SampleBufferCaptureEventStream`

`SampleBufferCaptureEventStream::start(...)` keeps `ReplayKit`'s existing typed capture bridge for setup, then hands sample events to an executor-agnostic bounded async stream.

## Platform notes

This crate targets macOS. APIs that only exist on iOS/tvOS are represented explicitly and return `ReplayKitError::NotSupported` instead of being omitted silently.

See [`COVERAGE.md`](COVERAGE.md) for the API-by-API matrix against the Apple SDK headers.

## Safety

All public APIs are safe Rust. The Swift bridge uses `strdup`/`free`-paired C strings and reference-counted opaque pointers; no raw memory is exposed through the public interface.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
