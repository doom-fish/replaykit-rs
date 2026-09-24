# Changelog

All notable changes to `replaykit-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Unreleased

### Security

- `RecordingObserver` and `DetailedRecordingObserver` freed their handler right after clearing the delegate, so a delegate callback already in flight called freed memory. Every observer now keeps its handler in a `doom_fish_utils` `CallbackContext` that the Swift side releases only after its last callback.
- Dropping a `BroadcastController` or `PreviewViewController` before its observer no longer hands Swift a dangling controller pointer.

### Fixed

- Observers no longer overwrite each other in `RPScreenRecorder`'s single delegate slot. One multiplexing delegate fans events out to every observer, and dropping an observer removes only that observer.
- `start_capture` delivers the captured sample buffers instead of per-buffer JSON summaries.
- A recording, capture, or clip buffering that starts after the 30 s blocking timeout (for example when the consent prompt is approved late) is stopped again, so `TimedOut` always means nothing was started.
- AppKit objects are no longer touched off the main thread, including from `Debug`.

### Changed

- **Breaking:** `CaptureSample` holds `sample_buffer: apple_cf::cm::CMSampleBuffer` next to `sample_type` and `video_orientation`, replacing the `num_samples`, `data_is_ready`, `presentation_time_seconds`, and `duration_seconds` fields. `CaptureSample` and `CaptureEvent` now derive `Eq`.
- **Breaking:** Observer handlers (`ScreenRecorder::observe`, `ScreenRecorder::observe_detailed`, `PreviewViewController::observe`, `BroadcastController::observe`) and `SampleBufferDelegate` implementations must be `Send + Sync`.
- **Breaking:** `CameraPreviewView`, `PreviewViewController`, and `PreviewViewControllerObserver` are `!Send` and `!Sync`. `ScreenRecorder::camera_preview_view`, `CameraPreviewView::is_hidden`, `PreviewViewController::is_view_loaded`, `PreviewViewController::observe`, and `PreviewEventStream::observe` return `Result` and fail with `ReplayKitError::MainThreadRequired` off the main thread. Their `Debug` output only contains the class name.
- **Breaking:** `ScreenRecorder::stop_recording_with_preview`, `AsyncScreenRecorder::stop_recording`, and `DetailedRecordingEvent::DidStopRecording` return preview controllers as `PreviewViewControllerHandle`.
- **Breaking:** `BroadcastExtensionContext`, `BroadcastHandler`, and `BroadcastSampleHandler` no longer create standalone objects that `ReplayKit` ignores. Code running inside a broadcast extension passes its own `NSExtensionContext`, `RPBroadcastHandler`, or `RPBroadcastSampleHandler` to the new `unsafe fn from_raw_borrowed`, which checks the class, retains the object, and returns `InvalidArgument` for null or mismatched objects.
- **Breaking:** `BroadcastSampleHandler` wraps the extension's own sample handler: `update_service_info`, `update_broadcast_url`, and `class_name` moved to `as_handler()`, and `finish_broadcast_with_error` stays. `processSampleBuffer:withType:` and the lifecycle hooks are still delivered only to the extension's Swift or Objective-C subclass.
- **Breaking:** The async recording futures report errors like the synchronous calls: framework errors arrive as `ReplayKitError::Framework` with their domain and code, and bridge errors keep their variant, instead of every failure becoming `ReplayKitError::Unknown(localizedDescription)`.
- **Breaking:** `AsyncScreenRecorder::stop_recording_with_output` takes `impl AsRef<Path>`, like `ScreenRecorder::stop_recording_to_output`, and resolves to `InvalidArgument` for paths that are not UTF-8 or contain NUL. It used to take a `&CStr` whose invalid UTF-8 was silently replaced.
- **Breaking:** `RP_RECORDING_ERROR_DOMAIN` and `SC_STREAM_ERROR_DOMAIN` hold the framework's values (`com.apple.ReplayKit.RPRecordingErrorDomain` and `com.apple.ScreenCaptureKit.SCStreamErrorDomain`) instead of the symbol names, which never matched a real error's domain. `ReplayKitFrameworkError::recording_code` only maps codes in `RP_RECORDING_ERROR_DOMAIN` and is no longer `const`.
- **Breaking:** When the broadcast picker returns neither a controller nor an error, the result is `ReplayKitError::Unknown` instead of a fabricated `Framework` error with code -1.
- **Breaking:** `SystemBroadcastPickerView`, `BroadcastConfiguration`, and `BroadcastActivityViewController` are uninhabited enums without `Default`, so their constructors can only return `NotSupported`.
- **Breaking:** The minimum macOS version is 12.0 (was 11.0). The Swift bridge uses Swift concurrency, which only ships with the OS from macOS 12, so `Package.swift`, the link minimum and the README now say 12.0, and clip buffering lost its pre-12 `NotSupported` path.
- Preview handles and AppKit wrappers dropped off the main thread release their object on the main queue.
- Requires `doom-fish-utils` `>=0.4.1, <0.5` and the new `apple-cf` `>=0.11, <0.12` dependency; `rust-version` is now 1.82.

### Added

- `PreviewViewControllerHandle`, a `Send + Sync` handle whose `to_controller()` opens a `PreviewViewController` on the main thread.
- `ReplayKitError::MainThreadRequired`.
- `BroadcastExtensionContext::from_raw_borrowed`, `BroadcastHandler::from_raw_borrowed`, `BroadcastSampleHandler::from_raw_borrowed`, and `BroadcastSampleHandler::as_handler`.

### Removed

- **Breaking:** `BroadcastExtensionContext::new`, `BroadcastHandler::new`, `BroadcastSampleHandler::new`, and their `Default` impls, which created standalone objects.
- **Breaking:** `BroadcastSampleHandler`'s `broadcast_started`, `broadcast_started_with_setup_info`, `broadcast_paused`, `broadcast_resumed`, `broadcast_finished`, and `broadcast_annotated_with_application_info`. They invoked lifecycle hooks that `ReplayKit` calls on the extension's subclass.
- The hidden, never-constructed `AsyncStartCapture` and `AsyncStopCapture` futures and the unused asynchronous capture exports behind them.

## [0.4.2] - 2026-06-06

- Hardened the Swift bridge against use-after-free of observer and capture contexts, and contained Rust panics before they cross the FFI boundary, including in the broadcast-picker completion.

## [0.4.1] - 2026-05-20

- Migrated local `take_string` body to call `doom_fish_utils::ffi_string::take_owned_cstring_c`. Centralises the duplicated FFI take-string pattern fleet-wide. No public API change.

## [0.4.0] - 2026-05-20

### Added

- `AsyncBroadcastActivityControllerHandle::show(...)` plus bounded async streams for `BroadcastController`, `PreviewViewController`, typed `ScreenRecorder` delegate events, and sample-buffer capture events.
- `AsyncScreenRecorder::detailed_events(...)` and `AsyncScreenRecorder::capture_events(...)` so delegate-driven ReplayKit flows can be consumed without blocking the async caller.

### Notes

- Phase 32 completeness + async sweep.

## [0.3.5] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.3.4] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.3.3] - 2026-05-19

### Changed

- Documented `RPBroadcastActivityViewControllerDelegate` as an explicit iOS/tvOS-only audit exemption after re-checking `RPBroadcast.h`; the crate continues to expose the macOS-native `RPBroadcastActivityController` flow instead.

## [0.3.2] - 2026-05-18

- Widen doom-fish-utils version bound to `<0.3` so 0.2.x resolves.

## [0.3.1] - 2026-05-17

### Fixed

- Fixed FFI panic safety: wrapped extern "C" callbacks in `catch_user_panic` to prevent unwinding into Swift
- Added explicit `SAFETY` comments to all unsafe blocks in `async_api` module explaining pointer validity and lifetime guarantees
- Added explicit `Send + Sync` trait implementations for async Future types with documentation


## [0.3.0] - 2026-05-17

### Added

- Added `async_api` module (gated by `async` feature) providing async Future-based wrappers for ReplayKit operations
- Async operations:
  - `AsyncScreenRecorder::start_recording()` — non-blocking async start recording
  - `AsyncScreenRecorder::stop_recording()` — non-blocking async stop recording with optional preview
  - `AsyncScreenRecorder::stop_recording_with_output()` — non-blocking async stop with file output
  - `AsyncScreenRecorder::discard_recording()` — non-blocking async discard recording
- Uses executor-agnostic design (works with any async runtime: Tokio, async-std, smol, etc.)
- Integrated `doom-fish-utils` completion pattern for true async operations (no blocking)
- Added example `01_async_recording.rs` demonstrating async API usage with `pollster`
- Added `tests/async_api_tests.rs` for async operation testing
- Note: `startCapture` and `stopCapture` streaming APIs deferred to Tier-2 Stream implementation

## [0.2.1] - 2026-05-16

### Added

- Added `BroadcastExtensionContext` for `NSExtensionContext` broadcast-extension helpers, including `load_broadcasting_application_info` and `complete_request_with_broadcast_url`
- Added `BroadcastHandler`, `BroadcastSampleHandler`, and the `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY` constant for ReplayKit broadcast-extension authoring APIs
- Added broadcast-extension integration tests and the `08_rp_broadcast_extension_support` example

## [0.2.0] - 2026-05-16

### Added

- Split the Swift bridge and Rust FFI into per-area files for screen recording, broadcast control, preview UI, sample-buffer capture, and explicit unsupported macOS stubs
- Extended `ScreenRecorder` with structured state snapshots, microphone/camera/camera-position accessors, camera preview views, preview-controller returns, direct-to-file recording, clip buffering, and detailed delegate forwarding
- Added `PreviewViewController`, `BroadcastControllerObserver`, `SampleBufferCaptureSession`, `SampleBufferType`, and typed `RPRecordingErrorCode` support
- Added explicit macOS `NotSupported` wrappers for `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration`
- Added numbered examples, per-area integration tests, and `COVERAGE.md`

## [0.1.0] - 2025-01-01

### Added

- `ScreenRecorder::shared()` wrapping `RPScreenRecorder.shared()`
- State getters: `is_available`, `is_recording`, `is_microphone_enabled`, `is_camera_enabled`
- `start_recording` / `stop_recording` blocking wrappers
- `ScreenRecorder::observe` returning a `RecordingObserver` RAII delegate guard
- `BroadcastActivityControllerHandle::show` for the macOS broadcast picker
- `BroadcastController` with `start`, `finish`, `pause`, `resume`, `broadcast_url`
- Multi-file Swift bridge (`Core.swift`, `ScreenRecorder.swift`, `Broadcast.swift`)
