# ReplayKit.framework coverage for `replaykit-rs` v0.2.0

Legend:

- ✅ implemented
- 🟡 partial
- ⏭️ skipped — unavailable / deprecated / extension-only

Notes:

- The requested **RPPreviewView** area maps to Apple's `RPPreviewViewController` on macOS.
- The requested **RPBroadcastActivityViewController** area maps to macOS `RPBroadcastActivityController`; the iOS view-controller type is surfaced explicitly as `NotSupported`.
- The requested **RPSampleBufferDelegate** area is implemented through `RPScreenRecorder.startCapture` + `SampleBufferCaptureSession` + `SampleBufferType`.

## ReplayKit.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Umbrella imports (`RPPreviewViewController`, `RPScreenRecorder`, `RPBroadcast`, `RPBroadcastExtension`, `RPError`) | ✅ | crate root / module re-exports | Header coverage tracked below |

## RPScreenRecorder.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPScreenRecorder.sharedRecorder` | ✅ | `ScreenRecorder::shared` | Shared singleton handle |
| `-startRecordingWithMicrophoneEnabled:handler:` | ⏭️ skipped | — | Unavailable on macOS |
| `-startRecordingWithHandler:` | ✅ | `ScreenRecorder::start_recording` | Blocking bridge with typed errors |
| `-stopRecordingWithHandler:` | ✅ | `ScreenRecorder::stop_recording`, `stop_recording_with_preview` | Preserves preview controller when requested |
| `-stopRecordingWithOutputURL:completionHandler:` | ✅ | `ScreenRecorder::stop_recording_to_output` | macOS 11+ |
| `-discardRecordingWithHandler:` | ✅ | `ScreenRecorder::discard_recording` | Blocking bridge |
| `-startCaptureWithHandler:completionHandler:` | ✅ | `ScreenRecorder::start_capture`, `SampleBufferCaptureSession` | Typed sample-buffer events |
| `-stopCaptureWithHandler:` | ✅ | `SampleBufferCaptureSession::stop` / `Drop` | Blocking bridge |
| `-startClipBufferingWithCompletionHandler:` | ✅ | `ScreenRecorder::start_clip_buffering` | Returns `NotSupported` before macOS 12 |
| `-stopClipBufferingWithCompletionHandler:` | ✅ | `ScreenRecorder::stop_clip_buffering` | Returns `NotSupported` before macOS 12 |
| `-exportClipToURL:duration:completionHandler:` | ✅ | `ScreenRecorder::export_clip_to_output` | Returns `NotSupported` before macOS 12 |
| `delegate` | ✅ | `ScreenRecorder::observe`, `observe_detailed` | Lightweight + typed delegate bridges |
| `available` | ✅ | `ScreenRecorder::is_available`, `ScreenRecorder::state` | |
| `recording` | ✅ | `ScreenRecorder::is_recording`, `ScreenRecorder::state` | |
| `microphoneEnabled` | ✅ | `is_microphone_enabled`, `set_microphone_enabled` | |
| `cameraEnabled` | ✅ | `is_camera_enabled`, `set_camera_enabled` | |
| `cameraPosition` | ✅ | `camera_position`, `set_camera_position`, `CameraPosition` | |
| `cameraPreviewView` | ✅ | `camera_preview_view`, `CameraPreviewView` | Exposed as retained `NSView` wrapper |
| Deprecated `screenRecorder:didStopRecordingWithError:previewViewController:` | ⏭️ skipped | — | Unavailable on macOS |
| `screenRecorder:didStopRecordingWithPreviewViewController:error:` | ✅ | `observe_detailed`, `stop_recording_with_preview` | Preview controller surfaced explicitly |
| `screenRecorderDidChangeAvailability:` | ✅ | `observe`, `observe_detailed` | |

## RPPreviewViewController.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPPreviewViewController` | ✅ | `PreviewViewController` | Requested “RPPreviewView” area |
| `previewControllerDelegate` | ✅ | `PreviewViewController::observe` | |
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
| `NSExtensionContext.loadBroadcastingApplicationInfoWithCompletion:` | ⏭️ skipped | — | Broadcast UI extension-only API |
| Deprecated `completeRequestWithBroadcastURL:broadcastConfiguration:setupInfo:` | ⏭️ skipped | — | Unavailable on macOS |
| `completeRequestWithBroadcastURL:setupInfo:` | ⏭️ skipped | — | Broadcast UI extension-only API |
| `RPBroadcastHandler` | ⏭️ skipped | — | Upload-extension subclass API |
| `updateServiceInfo:` | ⏭️ skipped | — | Upload-extension callback API |
| `updateBroadcastURL:` | ⏭️ skipped | — | Upload-extension callback API |
| `RPBroadcastMP4ClipHandler` | ⏭️ skipped | — | Unavailable on macOS |
| `RPSampleBufferType` | ✅ | `SampleBufferType` | |
| `RPVideoSampleOrientationKey` | ✅ | `CaptureSample::video_orientation` | Raw attachment value is forwarded |
| `RPApplicationInfoBundleIdentifierKey` | ⏭️ skipped | — | Broadcast annotation key for upload extensions |
| `RPBroadcastSampleHandler` | ⏭️ skipped | — | Upload-extension subclass API |
| `broadcastStartedWithSetupInfo:` | ⏭️ skipped | — | Upload-extension subclass API |
| `broadcastPaused` | ⏭️ skipped | — | Upload-extension subclass API |
| `broadcastResumed` | ⏭️ skipped | — | Upload-extension subclass API |
| `broadcastFinished` | ⏭️ skipped | — | Upload-extension subclass API |
| `broadcastAnnotatedWithApplicationInfo:` | ⏭️ skipped | — | iOS-only upload-extension API |
| `processSampleBuffer:withType:` | ⏭️ skipped | — | Upload-extension subclass API |
| `finishBroadcastWithError:` | ⏭️ skipped | — | Upload-extension subclass API |

## RPError.h

| API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `RPRecordingErrorDomain` | ✅ | `RP_RECORDING_ERROR_DOMAIN`, `ReplayKitFrameworkError::domain` | |
| `SCStreamErrorDomain` | ✅ | `SC_STREAM_ERROR_DOMAIN` | Re-exported constant |
| `RPRecordingErrorCode` enum | ✅ | `RecordingErrorCode` | Full code mapping from header |
