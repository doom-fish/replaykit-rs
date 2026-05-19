# Changelog

## [0.3.4] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.3.3] - 2026-05-19

### Changed

- Documented `RPBroadcastActivityViewControllerDelegate` as an explicit iOS/tvOS-only audit exemption after re-checking `RPBroadcast.h`; the crate continues to expose the macOS-native `RPBroadcastActivityController` flow instead.

## [0.3.2] - 2026-05-18

- Widen doom-fish-utils version bound to `<0.3` so 0.2.x resolves.

All notable changes to `replaykit-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
