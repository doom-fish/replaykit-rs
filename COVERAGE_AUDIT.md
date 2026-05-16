# replaykit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 18
VERIFIED: 14
GAPS: 4
EXEMPT: 0
COVERAGE_PCT: 77.8%

Notes:

- This audit follows the shared rubric from `audit-instructions.md`: only macOS-available top-level `@interface`, defined `@protocol`, enum typedef, category, and exported constant symbols are counted.
- iOS/tvOS-only or `API_UNAVAILABLE(macos)` symbols such as `RPBroadcastActivityViewController`, `RPSystemBroadcastPickerView`, and `RPBroadcastConfiguration` are filtered out rather than counted as gaps, even though the crate exposes explicit `NotSupported` wrappers for them.
- The remaining gaps are all broadcast-extension authoring APIs from `RPBroadcastExtension.h`; the app-side recording, preview, and broadcast-control surface is covered.
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
| `RPSampleBufferType` | enum | `RPBroadcastExtension.h` | `SampleBufferType`, `CaptureSample::sample_type` |
| `RPVideoSampleOrientationKey` | constant | `RPBroadcastExtension.h` | `CaptureSample::video_orientation` |
| `RPRecordingErrorDomain` | constant | `RPError.h` | `RP_RECORDING_ERROR_DOMAIN` |
| `SCStreamErrorDomain` | constant | `RPError.h` | `SC_STREAM_ERROR_DOMAIN` |
| `RPRecordingErrorCode` | enum | `RPError.h` | `RecordingErrorCode` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| `NSExtensionContext (RPBroadcastExtension)` | category | `RPBroadcastExtension.h` | Broadcast UI extension methods (`loadBroadcastingApplicationInfo…`, `completeRequestWithBroadcastURL…`) are not wrapped; the crate only targets app-side APIs. |
| `RPBroadcastHandler` | class | `RPBroadcastExtension.h` | Upload-extension base class is not exposed. |
| `RPApplicationInfoBundleIdentifierKey` | constant | `RPBroadcastExtension.h` | Annotation key used by broadcast-extension metadata is not surfaced. |
| `RPBroadcastSampleHandler` | class | `RPBroadcastExtension.h` | Sample-buffer broadcast extension subclass API is not exposed. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |

No macOS-deprecated public ReplayKit symbols met the audit criteria.
