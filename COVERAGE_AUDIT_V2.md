# replaykit-rs coverage audit v2 (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 18
VERIFIED: 18
GAPS: 0
EXEMPT: 4
COVERAGE_PCT: 100.0

Audit methodology: Walked ReplayKit umbrella header and all constituent headers (RPScreenRecorder.h, RPPreviewViewController.h, RPBroadcast.h, RPBroadcastExtension.h, RPError.h, RPBroadcastConfiguration.h) to enumerate public @interface, @protocol, typedef enum, and extern const symbols. Filtered out symbols with `API_UNAVAILABLE(macos)`, iOS-only annotations without macOS counterpart, and deprecated-without-macOS items. The framework surface is primarily iOS; ReplayKit on macOS (11.0+) covers screen recording, preview, broadcast control, and broadcast extension handling. All 18 macOS-available symbols are verified wrapped by the crate's Rust safe API.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| `RPCameraPosition` | enum | `RPScreenRecorder.h` | `CameraPosition` |
| `RPScreenRecorder` | class | `RPScreenRecorder.h` | `ScreenRecorder` |
| `RPScreenRecorderDelegate` | protocol | `RPScreenRecorder.h` | `ScreenRecorder::observe`, `ScreenRecorder::observe_detailed`, `RecordingEvent`, `DetailedRecordingEvent` |
| `RPPreviewViewController` | class | `RPPreviewViewController.h` | `PreviewViewController` |
| `RPPreviewViewControllerDelegate` | protocol | `RPPreviewViewController.h` | `PreviewViewController::observe`, `PreviewEvent` |
| `RPBroadcastActivityController` | class | `RPBroadcast.h` | `BroadcastActivityControllerHandle::show` |
| `RPBroadcastActivityControllerDelegate` | protocol | `RPBroadcast.h` | `BroadcastActivityControllerHandle::show` callback |
| `RPBroadcastController` | class | `RPBroadcast.h` | `BroadcastController` |
| `RPBroadcastControllerDelegate` | protocol | `RPBroadcast.h` | `BroadcastController::observe`, `BroadcastControllerEvent` |
| `NSExtensionContext (RPBroadcastExtension)` | category | `RPBroadcastExtension.h` | `BroadcastExtensionContext` |
| `RPBroadcastHandler` | class | `RPBroadcastExtension.h` | `BroadcastHandler` |
| `RPSampleBufferType` | enum | `RPBroadcastExtension.h` | `SampleBufferType`, `CaptureSample::sample_type` |
| `RPVideoSampleOrientationKey` | constant | `RPBroadcastExtension.h` | `CaptureSample::video_orientation` |
| `RPApplicationInfoBundleIdentifierKey` | constant | `RPBroadcastExtension.h` | `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY` |
| `RPBroadcastSampleHandler` | class | `RPBroadcastExtension.h` | `BroadcastSampleHandler` |
| `RPRecordingErrorDomain` | constant | `RPError.h` | `RP_RECORDING_ERROR_DOMAIN` |
| `SCStreamErrorDomain` | constant | `RPError.h` | `SC_STREAM_ERROR_DOMAIN` |
| `RPRecordingErrorCode` | enum | `RPError.h` | `RecordingErrorCode` |

## 🔴 GAPS

No gaps identified. All macOS-available public ReplayKit symbols are covered by the crate.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `RPBroadcastActivityViewController` | class | `RPBroadcast.h` | iOS-only variant; macOS uses RPBroadcastActivityController instead | `API_AVAILABLE(ios(10.0), tvos(10.0))` — no macOS annotation |
| `RPBroadcastActivityViewControllerDelegate` | protocol | `RPBroadcast.h` | iOS-only variant; paired with RPBroadcastActivityViewController | `API_AVAILABLE(ios(10.0), tvos(10.0))` — no macOS annotation |
| `RPSystemBroadcastPickerView` | class | `RPBroadcast.h` | iOS-only; Control Center broadcast picker | `API_UNAVAILABLE(tvos, macos)` |
| `RPBroadcastMP4ClipHandler` | class | `RPBroadcastExtension.h` | Deprecated and macOS-unavailable; replaced by RPBroadcastSampleHandler | `API_DEPRECATED(...) API_UNAVAILABLE(macos)` |
