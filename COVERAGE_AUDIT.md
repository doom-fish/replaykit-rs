# replaykit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 18
VERIFIED: 18
GAPS: 0
EXEMPT: 0
COVERAGE_PCT: 100.0%

Notes:

- This audit follows the shared rubric from `audit-instructions.md`: only macOS-available top-level `@interface`, defined `@protocol`, enum typedef, category, and exported constant symbols are counted.
- iOS/tvOS-only or `API_UNAVAILABLE(macos)` symbols such as `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration` are filtered out rather than counted as gaps, even though the crate exposes explicit `NotSupported` wrappers for them.
- Additional explicitly documented unavailable symbols may appear below for completeness; they are excluded from the macOS coverage denominator above.
- `RPBroadcastExtension.h` is now covered through `BroadcastExtensionContext`, `BroadcastHandler`, `BroadcastSampleHandler`, and `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY` alongside the existing app-side recording, preview, and broadcast-control surface.
- `COVERAGE.md` remains the member-by-member matrix; this file is the top-level symbol audit.

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

No remaining gaps.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

No macOS-deprecated public ReplayKit symbols met the audit criteria.

### Explicit additional symbols (excluded from counts)

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `RPBroadcastActivityViewControllerDelegate` | protocol | `RPBroadcast.h` | iOS/tvOS-only delegate for `RPBroadcastActivityViewController`; macOS uses `RPBroadcastActivityControllerDelegate`, which is already wrapped. | `API_AVAILABLE(ios(10.0), tvos(10.0))` |
