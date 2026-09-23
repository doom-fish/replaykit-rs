# replaykit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 18
VERIFIED: 17
GAPS: 1
EXEMPT: 0
COVERAGE_PCT: 94.4%

Notes:

- The numbers count top-level symbols only. A symbol is VERIFIED when a Rust type or function wraps it; they say nothing about member coverage or about whether the wrapped object can be used outside an extension. `COVERAGE.md` has the member matrix, including the partial rows.
- The symbol list comes from MacOSX26.2.sdk and has not been regenerated. The ReplayKit headers in MacOSX26.5.sdk declare the same 18 macOS symbols (checked for replaykit-rs 0.5.0).
- `NSExtensionContext (RPBroadcastExtension)` and `RPBroadcastHandler` are wrapped, but only for objects the crate creates itself; ReplayKit ignores calls on them outside an extension.
- This audit follows the shared rubric from `audit-instructions.md`: only macOS-available top-level `@interface`, defined `@protocol`, enum typedef, category, and exported constant symbols are counted.
- iOS/tvOS-only or `API_UNAVAILABLE(macos)` symbols such as `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration` are filtered out rather than counted as gaps, even though the crate exposes explicit `NotSupported` wrappers for them.
- Additional explicitly documented unavailable symbols may appear below for completeness; they are excluded from the macOS coverage denominator above.
- `RPBroadcastExtension.h` is covered through `BroadcastExtensionContext`, `BroadcastHandler`, and `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY`; `RPBroadcastSampleHandler` is a gap because a broadcast upload extension needs an Objective-C principal-class subclass that this crate cannot provide.
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
| `RPRecordingErrorDomain` | constant | `RPError.h` | `RP_RECORDING_ERROR_DOMAIN` |
| `SCStreamErrorDomain` | constant | `RPError.h` | `SC_STREAM_ERROR_DOMAIN` |
| `RPRecordingErrorCode` | enum | `RPError.h` | `RecordingErrorCode` |

## 🔴 GAPS
| Symbol | Kind | Header | Reason |
| --- | --- | --- | --- |
| `RPBroadcastSampleHandler` | class | `RPBroadcastExtension.h` | Only usable as the principal class of a broadcast upload extension; `processSampleBuffer:withType:` never reaches Rust, so `BroadcastSampleHandler` returns `NotSupported` |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

No macOS-deprecated public ReplayKit symbols met the audit criteria.

### Explicit additional symbols (excluded from counts)

| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `RPBroadcastActivityViewControllerDelegate` | protocol | `RPBroadcast.h` | iOS/tvOS-only delegate for `RPBroadcastActivityViewController`; macOS uses `RPBroadcastActivityControllerDelegate`, which is already wrapped. | `API_AVAILABLE(ios(10.0), tvos(10.0))` |
