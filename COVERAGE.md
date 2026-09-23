# ReplayKit.framework coverage for `replaykit-rs` v0.5.0

Legend:

- ✅ implemented
- 🟡 partial
- ❌ not supported — available on macOS, but this crate cannot drive it
- ⏭️ skipped — unavailable / deprecated / extension-only

Notes:

- The requested **RPPreviewView** area maps to Apple's `RPPreviewViewController` on macOS.
- The requested **RPBroadcastActivityViewController** area maps to macOS `RPBroadcastActivityController`; the iOS view-controller type is surfaced explicitly as `NotSupported`.
- The requested **RPSampleBufferDelegate** area is implemented through `RPScreenRecorder.startCapture` + `SampleBufferCaptureSession` + `SampleBufferType`; each buffer is delivered as a retained `apple_cf::cm::CMSampleBuffer`.
- `BroadcastExtensionContext` and `BroadcastHandler` wrap `NSExtensionContext` and `RPBroadcastHandler` objects created by the crate. ReplayKit only acts on the instances it creates inside a broadcast extension, so these rows are partial.
- Broadcast upload extensions are not supported: ReplayKit delivers samples to an `RPBroadcastSampleHandler` subclass that is the extension's principal class, which this crate cannot provide. `BroadcastSampleHandler` reports `NotSupported`.

## ReplayKit.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Umbrella imports (`RPPreviewViewController`, `RPScreenRecorder`, `RPBroadcast`, `RPBroadcastExtension`, `RPError`) | ✅ | crate root / module re-exports | Header coverage tracked below |

## RPScreenRecorder.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPScreenRecorder.sharedRecorder` | ✅ | `ScreenRecorder::shared` | Shared singleton handle |
| `-startRecordingWithMicrophoneEnabled:handler:` | ⏭️ skipped | — | Unavailable on macOS |
| `-startRecordingWithHandler:` | ✅ | `ScreenRecorder::start_recording` | Blocking bridge with typed errors; a recording that starts after the 30 s timeout is stopped and discarded |
| `-stopRecordingWithHandler:` | ✅ | `ScreenRecorder::stop_recording`, `stop_recording_with_preview` | Preserves preview controller when requested |
| `-stopRecordingWithOutputURL:completionHandler:` | ✅ | `ScreenRecorder::stop_recording_to_output` | macOS 11+ |
| `-discardRecordingWithHandler:` | ✅ | `ScreenRecorder::discard_recording` | Blocking bridge |
| `-startCaptureWithHandler:completionHandler:` | ✅ | `ScreenRecorder::start_capture`, `SampleBufferCaptureSession` | Retained `CMSampleBuffer` per video/audio buffer; a capture that starts after the 30 s timeout is stopped |
| `-stopCaptureWithHandler:` | ✅ | `SampleBufferCaptureSession::stop` / `Drop` | Blocking bridge |
| `-startClipBufferingWithCompletionHandler:` | ✅ | `ScreenRecorder::start_clip_buffering` | Returns `NotSupported` before macOS 12; stopped again if it starts after the 30 s timeout |
| `-stopClipBufferingWithCompletionHandler:` | ✅ | `ScreenRecorder::stop_clip_buffering` | Returns `NotSupported` before macOS 12 |
| `-exportClipToURL:duration:completionHandler:` | ✅ | `ScreenRecorder::export_clip_to_output` | Returns `NotSupported` before macOS 12 |
| `delegate` | ✅ | `ScreenRecorder::observe`, `observe_detailed` | One multiplexing delegate shared by every observer; dropping an observer removes only that observer |
| `available` | ✅ | `ScreenRecorder::is_available`, `ScreenRecorder::state` | |
| `recording` | ✅ | `ScreenRecorder::is_recording`, `ScreenRecorder::state` | |
| `microphoneEnabled` | ✅ | `is_microphone_enabled`, `set_microphone_enabled` | |
| `cameraEnabled` | ✅ | `is_camera_enabled`, `set_camera_enabled` | |
| `cameraPosition` | ✅ | `camera_position`, `set_camera_position`, `CameraPosition` | |
| `cameraPreviewView` | ✅ | `camera_preview_view`, `CameraPreviewView` | Main-thread-only retained `NSView` wrapper |
| Deprecated `screenRecorder:didStopRecordingWithError:previewViewController:` | ⏭️ skipped | — | Unavailable on macOS |
| `screenRecorder:didStopRecordingWithPreviewViewController:error:` | ✅ | `observe_detailed`, `stop_recording_with_preview` | Preview controller surfaced as a `PreviewViewControllerHandle` |
| `screenRecorderDidChangeAvailability:` | ✅ | `observe`, `observe_detailed` | |

## RPPreviewViewController.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPPreviewViewController` | ✅ | `PreviewViewControllerHandle`, `PreviewViewController` | Requested “RPPreviewView” area; the controller is main-thread-only |
| `previewControllerDelegate` | ✅ | `PreviewViewController::observe` | Main thread only |
| `mode` | ⏭️ skipped | — | tvOS-only |
| `previewControllerDidFinish:` | ✅ | `PreviewEvent::DidFinish` | |
| `previewController:didFinishWithActivityTypes:` | ✅ | `PreviewEvent::DidFinishWithActivityTypes` | |

## RPBroadcast.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPBroadcastActivityController` | ✅ | `BroadcastActivityControllerHandle::show` | macOS broadcast picker flow |
| `RPBroadcastActivityController.delegate` | ✅ | Swift delegate bridge | Internal forwarding to `ShowResult` |
| `broadcastActivityController:didFinishWithBroadcastController:error:` | ✅ | `BroadcastActivityControllerHandle::show` callback | |
| `RPBroadcastActivityViewController` | ⏭️ skipped | `BroadcastActivityViewController` | iOS/tvOS-only; explicit `NotSupported` wrapper |
| `+loadBroadcastActivityViewControllerWithHandler:` | ⏭️ skipped | — | iOS/tvOS-only |
| `+loadBroadcastActivityViewControllerWithPreferredExtension:handler:` | ⏭️ skipped | — | iOS-only |
| `RPBroadcastController.broadcasting` | ✅ | `BroadcastController::is_broadcasting` | |
| `RPBroadcastController.paused` | ✅ | `BroadcastController::is_paused` | |
| `RPBroadcastController.broadcastURL` | ✅ | `BroadcastController::broadcast_url` | |
| `RPBroadcastController.serviceInfo` | ✅ | `BroadcastController::service_info` | JSON-encoded bridge |
| `RPBroadcastController.delegate` | ✅ | `BroadcastController::observe` | |
| `RPBroadcastController.broadcastExtensionBundleID` | ⏭️ skipped | — | Unavailable on macOS |
| `-startBroadcastWithHandler:` | ✅ | `BroadcastController::start` | |
| `-pauseBroadcast` | ✅ | `BroadcastController::pause` | |
| `-resumeBroadcast` | ✅ | `BroadcastController::resume` | |
| `-finishBroadcastWithHandler:` | ✅ | `BroadcastController::finish` | |
| `broadcastController:didFinishWithError:` | ✅ | `BroadcastControllerEvent::DidFinish` | |
| `broadcastController:didUpdateServiceInfo:` | ✅ | `BroadcastControllerEvent::DidUpdateServiceInfo` | |
| `broadcastController:didUpdateBroadcastURL:` | 🟡 partial | `BroadcastControllerEvent::DidUpdateBroadcastUrl` | Apple marks it iOS/tvOS-only, but the bridge forwards the selector defensively if ReplayKit delivers it |
| `RPSystemBroadcastPickerView` | ⏭️ skipped | `SystemBroadcastPickerView` | iOS-only; explicit `NotSupported` wrapper |
| `preferredExtension` | ⏭️ skipped | `SystemBroadcastPickerView::new` | iOS-only |
| `showsMicrophoneButton` | ⏭️ skipped | `SystemBroadcastPickerView::new` | iOS-only |

## RPBroadcastConfiguration.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPBroadcastConfiguration` | ⏭️ skipped | `BroadcastConfiguration` | Deprecated and unavailable on macOS |
| `clipDuration` | ⏭️ skipped | — | Deprecated and unavailable on macOS |
| `videoCompressionProperties` | ⏭️ skipped | — | Deprecated and unavailable on macOS |

## RPBroadcastExtension.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `NSExtensionContext.loadBroadcastingApplicationInfoWithCompletion:` | 🟡 partial | `BroadcastExtensionContext::load_broadcasting_application_info` | Only on a context created by the crate; ReplayKit resolves it only for an extension-owned context |
| Deprecated `completeRequestWithBroadcastURL:broadcastConfiguration:setupInfo:` | ⏭️ skipped | — | Unavailable on macOS |
| `completeRequestWithBroadcastURL:setupInfo:` | 🟡 partial | `BroadcastExtensionContext::complete_request_with_broadcast_url`, `BroadcastExtensionContext::complete_request_with_broadcast_url_and_setup_info` | Standalone context; no effect outside an extension |
| `RPBroadcastHandler` | 🟡 partial | `BroadcastHandler` | Standalone instance created by the crate |
| `updateServiceInfo:` | 🟡 partial | `BroadcastHandler::update_service_info` | JSON is bridged to the ReplayKit dictionary type; no effect outside an extension |
| `updateBroadcastURL:` | 🟡 partial | `BroadcastHandler::update_broadcast_url` | No effect outside an extension |
| `RPBroadcastMP4ClipHandler` | ⏭️ skipped | — | Unavailable on macOS |
| `RPSampleBufferType` | ✅ | `SampleBufferType` | |
| `RPVideoSampleOrientationKey` | ✅ | `CaptureSample::video_orientation` | Raw attachment value is forwarded |
| `RPApplicationInfoBundleIdentifierKey` | ✅ | `RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY` | |
| `RPBroadcastSampleHandler` | ❌ not supported | `BroadcastSampleHandler` | Placeholder whose `new` returns `NotSupported`; an upload extension needs an Objective-C principal-class subclass |
| `broadcastStartedWithSetupInfo:` | ❌ not supported | — | Hook overridden by the extension's subclass |
| `broadcastPaused` | ❌ not supported | — | Hook overridden by the extension's subclass |
| `broadcastResumed` | ❌ not supported | — | Hook overridden by the extension's subclass |
| `broadcastFinished` | ❌ not supported | — | Hook overridden by the extension's subclass |
| `broadcastAnnotatedWithApplicationInfo:` | ❌ not supported | — | Hook overridden by the extension's subclass |
| `processSampleBuffer:withType:` | ❌ not supported | — | ReplayKit calls it on the extension's subclass; samples never reach Rust |
| `finishBroadcastWithError:` | ❌ not supported | — | Only meaningful on the extension's own sample handler |

## RPError.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPRecordingErrorDomain` | ✅ | `RP_RECORDING_ERROR_DOMAIN`, `ReplayKitFrameworkError::domain` | |
| `SCStreamErrorDomain` | ✅ | `SC_STREAM_ERROR_DOMAIN` | Re-exported constant |
| `RPRecordingErrorCode` enum | ✅ | `RecordingErrorCode` | Full code mapping from header |
