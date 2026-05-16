# replaykit-rs

Safe Rust bindings for Apple's **`ReplayKit`** framework on macOS.

[![Crates.io](https://img.shields.io/crates/v/replaykit-rs.svg)](https://crates.io/crates/replaykit-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

## Features

- `ScreenRecorder::shared()` — wraps `RPScreenRecorder.shared()`
- State getters: `is_available`, `is_recording`, `is_microphone_enabled`, `is_camera_enabled`
- `start_recording` / `stop_recording` — blocking wrappers with error propagation
- Delegate callbacks via `ScreenRecorder::observe` → `RecordingObserver` RAII guard
- `BroadcastActivityController` (macOS picker sheet) and `BroadcastController`

## Requirements

- macOS 11.0+
- Xcode with Swift toolchain installed

## Quick start

```rust
use replaykit::ScreenRecorder;

fn main() {
    let recorder = ScreenRecorder::shared().expect("ReplayKit unavailable");
    println!("available: {}", recorder.is_available());
}
```

## Safety

All public APIs are safe Rust.  The Swift bridge uses `strdup`/`free`-paired
C strings and reference-counted opaque pointers; no raw memory is exposed
through the public interface.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
