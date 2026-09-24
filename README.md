# replaykit-rs

Safe Rust bindings for Apple's **`ReplayKit`** framework on macOS.

[![Crates.io](https://img.shields.io/crates/v/replaykit-rs.svg)](https://crates.io/crates/replaykit-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

## Installation

```toml
[dependencies]
replaykit-rs = "0.5"
```

The library crate is named `replaykit`.

## Covered areas

- `RPScreenRecorder` state, microphone/camera controls, camera preview access, recording, direct-to-file recording, clip buffering, and typed delegate callbacks; any number of observers share the recorder's single delegate slot
- `RPBroadcastController` start/pause/resume/finish, `serviceInfo`, and delegate events
- macOS `RPBroadcastActivityController` through `BroadcastActivityControllerHandle::show`
- `RPPreviewViewController` delegate callbacks and support helpers
- `RPScreenRecorder.startCapture` via `SampleBufferCaptureSession`: every video and audio buffer arrives as a retained `apple_cf::cm::CMSampleBuffer` together with its `RPSampleBufferType` and `RPVideoSampleOrientationKey` value
- Code running inside a broadcast extension can hand its own `NSExtensionContext`, `RPBroadcastHandler`, or `RPBroadcastSampleHandler` to `BroadcastExtensionContext`, `BroadcastHandler`, or `BroadcastSampleHandler` through their `unsafe` `from_raw_borrowed` constructors, which check the class and retain the object. The wrappers load the broadcasting app's info, complete the setup request, update the service info and broadcast URL, and finish a broadcast with an error. `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY` names the bundle-identifier key
- Explicit `NotSupported` wrappers for macOS-unavailable `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration`. Their types are uninhabited, so the constructors can only return the error
- Typed `RPRecordingErrorCode` mapping plus replay/broadcast error domains
- **Async API**: executor-agnostic futures for recording + broadcast-picker flows, plus bounded async streams for broadcast-controller, preview-controller, detailed recorder, and sample-buffer capture events via the `async` feature

## Not supported

- Acting as a broadcast extension's principal class. `ReplayKit` calls `processSampleBuffer:withType:` and the broadcast lifecycle hooks on the `RPBroadcastSampleHandler` subclass named in the extension's `Info.plist`, and this crate cannot provide that subclass. Write it in Swift or Objective-C and pass it to Rust with `BroadcastSampleHandler::from_raw_borrowed`.

## Requirements

- macOS 12.0+
- Xcode with Swift toolchain installed

## Permissions and timeouts

Starting a recording, a capture, or clip buffering can show the system's screen-recording consent prompt. The blocking wrappers wait up to 30 seconds for `ReplayKit` and then return `ReplayKitError::TimedOut`. If the user approves after that, the crate stops the recording, capture, or clip buffering again, so a timeout always means nothing was started.

## Threading

- Every observer handler and capture delegate must be `Send + Sync`: `ReplayKit` calls them by shared reference from its own threads.
- `CameraPreviewView`, `PreviewViewController`, and `PreviewViewControllerObserver` wrap `AppKit` objects. They are `!Send` and `!Sync`, and every `AppKit` call returns `ReplayKitError::MainThreadRequired` off the main thread.
- Recorder APIs return preview controllers as `PreviewViewControllerHandle`, which can move between threads. Call `to_controller()` on the main thread to use it. Handles and wrappers release their object on the main queue.

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

All public APIs are safe Rust except the three `from_raw_borrowed` constructors: they take an Objective-C object pointer from code running inside a broadcast extension, and the caller must pass a live object (the class itself is checked). The Swift bridge uses `strdup`/`free`-paired C strings and reference-counted opaque pointers; no raw memory is exposed through the public interface.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
