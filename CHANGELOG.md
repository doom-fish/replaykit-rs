# Changelog

All notable changes to `replaykit-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
