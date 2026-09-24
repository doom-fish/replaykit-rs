//! Async API for `ReplayKit`.
//!
//! This module exposes executor-agnostic futures and bounded async streams for
//! the callback-shaped `ReplayKit` surfaces that the synchronous wrappers already
//! cover.
//!
//! ## Available types
//!
//! | Type | Wrapped surface |
//! |------|-----------------|
//! | [`AsyncScreenRecorder`] | one-shot recording helpers plus detailed/capture stream constructors |
//! | [`AsyncBroadcastActivityControllerHandle`] | macOS `RPBroadcastActivityController.showBroadcastPicker(...)` |
//! | [`BroadcastControllerEventStream`] | `RPBroadcastControllerDelegate` |
//! | [`PreviewEventStream`] | `RPPreviewViewControllerDelegate` |
//! | [`DetailedRecordingEventStream`] | typed `RPScreenRecorderDelegate` events |
//! | [`SampleBufferCaptureEventStream`] | `RPScreenRecorder.startCapture(...)` sample events |
//!
//! Streams use [`doom_fish_utils::stream::BoundedAsyncStream`]: they are
//! bounded, executor-agnostic, and drop the oldest buffered event when the
//! consumer falls behind.
//!
//! ## Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use replaykit::async_api::AsyncScreenRecorder;
//! use replaykit::prelude::*;
//!
//! let recorder = ScreenRecorder::shared().expect("ReplayKit unavailable");
//! AsyncScreenRecorder::start_recording(&recorder).await?;
//! println!("Recording started");
//! # Ok(())
//! # }
//! ```

use core::ffi::{c_char, c_void};
use std::future::{self, Future};
use std::path::Path;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};

use crate::broadcast_controller::{
    BroadcastController, BroadcastControllerEvent, BroadcastControllerObserver,
};
use crate::error::ReplayKitError;
use crate::ffi;
use crate::preview_view::{
    PreviewEvent, PreviewViewController, PreviewViewControllerHandle, PreviewViewControllerObserver,
};
use crate::private::{cstring_from_str, path_cstring, result_from_status, take_string};
use crate::sample_buffer_delegate::{CaptureEvent, SampleBufferCaptureSession};
use crate::screen_recorder::{DetailedRecordingEvent, DetailedRecordingObserver, ScreenRecorder};
use doom_fish_utils::completion::{AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::panic_safe::catch_user_panic;
use doom_fish_utils::stream::{BoundedAsyncStream, NextItem};

// ============================================================================
// Callback helpers
// ============================================================================

type RecordingCompletion<T> = AsyncCompletionFuture<Result<T, ReplayKitError>>;

fn flatten_completion<T>(
    result: Result<Result<T, ReplayKitError>, String>,
) -> Result<T, ReplayKitError> {
    result.map_err(ReplayKitError::Unknown).and_then(std::convert::identity)
}

extern "C" fn void_callback(
    _result: *const c_void,
    status: i32,
    error: *mut c_char,
    user_data: *mut c_void,
) {
    catch_user_panic("replaykit::async_api::void_callback", || {
        let outcome = result_from_status(status, error);
        // SAFETY: user_data comes from AsyncCompletion::create() and the Swift
        // bridge calls the completion at most once.
        unsafe { AsyncCompletion::<Result<(), ReplayKitError>>::complete_ok(user_data, outcome) };
    });
}

extern "C" fn preview_callback(
    result: *const c_void,
    status: i32,
    error: *mut c_char,
    user_data: *mut c_void,
) {
    catch_user_panic("replaykit::async_api::preview_callback", || {
        // SAFETY: a non-null result is a retained `RPPreviewViewController` supplied
        // by the Swift bridge for this completion.
        let preview = unsafe { PreviewViewControllerHandle::from_raw(result.cast_mut()) };
        let outcome = result_from_status(status, error).map(|()| preview);
        // SAFETY: user_data comes from AsyncCompletion::create() and the Swift
        // bridge calls the completion at most once.
        unsafe {
            AsyncCompletion::<Result<Option<PreviewViewControllerHandle>, ReplayKitError>>::complete_ok(
                user_data, outcome,
            );
        }
    });
}

extern "C" fn broadcast_activity_callback(
    user_data: *mut c_void,
    controller_ptr: *mut c_void,
    error_json: *mut c_char,
) {
    catch_user_panic("replaykit::async_api::broadcast_activity_callback", || {
        if error_json.is_null() && !controller_ptr.is_null() {
            // SAFETY: `controller_ptr` is retained by the Swift bridge for this callback.
            let controller = unsafe { BroadcastController::from_ptr(controller_ptr) };
            // SAFETY: user_data comes from AsyncCompletion::create() and the Swift
            // bridge calls the completion at most once.
            unsafe { AsyncCompletion::<BroadcastController>::complete_ok(user_data, controller) };
        } else {
            // SAFETY: `take_string` accepts null and frees bridge-owned C strings.
            let message = unsafe { take_string(error_json) }
                .unwrap_or_else(|| "unknown broadcast activity error".to_owned());
            // SAFETY: user_data comes from AsyncCompletion::create() and the Swift
            // bridge calls the completion at most once.
            unsafe { AsyncCompletion::<BroadcastController>::complete_err(user_data, message) };
        }
    });
}

// ============================================================================
// Future types
// ============================================================================

enum AsyncShowBroadcastActivityInner {
    Pending(AsyncCompletionFuture<BroadcastController>),
    Ready(future::Ready<Result<BroadcastController, ReplayKitError>>),
}

/// Future for the macOS broadcast activity picker.
pub struct AsyncShowBroadcastActivity {
    inner: AsyncShowBroadcastActivityInner,
}

impl AsyncShowBroadcastActivity {
    fn ready(result: Result<BroadcastController, ReplayKitError>) -> Self {
        Self {
            inner: AsyncShowBroadcastActivityInner::Ready(future::ready(result)),
        }
    }
}

// SAFETY: `AsyncShowBroadcastActivity` only contains futures over `BroadcastController`
// and `ReplayKitError`, both of which are `Send + Sync` for this crate.
unsafe impl Send for AsyncShowBroadcastActivity {}
// SAFETY: see `Send` above.
unsafe impl Sync for AsyncShowBroadcastActivity {}

impl Future for AsyncShowBroadcastActivity {
    type Output = Result<BroadcastController, ReplayKitError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match &mut this.inner {
            AsyncShowBroadcastActivityInner::Pending(future) => Pin::new(future)
                .poll(cx)
                .map(|result| result.map_err(|message| crate::error::from_message(&message))),
            AsyncShowBroadcastActivityInner::Ready(future) => Pin::new(future).poll(cx),
        }
    }
}

/// Future for async start recording operation.
pub struct AsyncStartRecording {
    inner: RecordingCompletion<()>,
}

impl Future for AsyncStartRecording {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(flatten_completion)
    }
}

/// Future for async stop recording operation (optionally returns a preview controller).
pub struct AsyncStopRecording {
    inner: RecordingCompletion<Option<PreviewViewControllerHandle>>,
}

impl Future for AsyncStopRecording {
    type Output = Result<Option<PreviewViewControllerHandle>, ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(flatten_completion)
    }
}

enum AsyncStopRecordingWithOutputInner {
    Pending(RecordingCompletion<()>),
    Ready(future::Ready<Result<(), ReplayKitError>>),
}

/// Future for async stop recording with file output.
pub struct AsyncStopRecordingWithOutput {
    inner: AsyncStopRecordingWithOutputInner,
}

impl Future for AsyncStopRecordingWithOutput {
    type Output = Result<(), ReplayKitError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.get_mut().inner {
            AsyncStopRecordingWithOutputInner::Pending(future) => {
                Pin::new(future).poll(cx).map(flatten_completion)
            }
            AsyncStopRecordingWithOutputInner::Ready(future) => Pin::new(future).poll(cx),
        }
    }
}

/// Future for async discard recording operation.
pub struct AsyncDiscardRecording {
    inner: RecordingCompletion<()>,
}

impl Future for AsyncDiscardRecording {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(flatten_completion)
    }
}

// ============================================================================
// Stream types
// ============================================================================

/// Bounded async stream of [`BroadcastControllerEvent`] values.
pub struct BroadcastControllerEventStream {
    inner: BoundedAsyncStream<BroadcastControllerEvent>,
    _observer: BroadcastControllerObserver,
}

impl BroadcastControllerEventStream {
    /// Register a bounded async stream backed by [`BroadcastController::observe`].
    #[must_use]
    pub fn observe(controller: &BroadcastController, capacity: usize) -> Self {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let observer = controller.observe(move |event| {
            catch_user_panic(
                "replaykit::async_api::broadcast_controller_event_stream",
                || {
                    sender.push(event);
                },
            );
        });

        Self {
            inner: stream,
            _observer: observer,
        }
    }

    /// Await the next buffered broadcast-controller event.
    pub const fn next(&self) -> NextItem<'_, BroadcastControllerEvent> {
        self.inner.next()
    }

    /// Try to pop the next buffered broadcast-controller event without waiting.
    pub fn try_next(&self) -> Option<BroadcastControllerEvent> {
        self.inner.try_next()
    }

    /// Number of buffered broadcast-controller events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }
}

impl std::fmt::Debug for BroadcastControllerEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastControllerEventStream")
            .field("buffered_count", &self.buffered_count())
            .finish_non_exhaustive()
    }
}

/// Bounded async stream of [`PreviewEvent`] values.
pub struct PreviewEventStream {
    inner: BoundedAsyncStream<PreviewEvent>,
    _observer: PreviewViewControllerObserver,
}

impl PreviewEventStream {
    /// Register a bounded async stream backed by [`PreviewViewController::observe`].
    pub fn observe(
        controller: &PreviewViewController,
        capacity: usize,
    ) -> Result<Self, ReplayKitError> {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let observer = controller.observe(move |event| {
            catch_user_panic("replaykit::async_api::preview_event_stream", || {
                sender.push(event);
            });
        })?;

        Ok(Self {
            inner: stream,
            _observer: observer,
        })
    }

    /// Await the next buffered preview event.
    pub const fn next(&self) -> NextItem<'_, PreviewEvent> {
        self.inner.next()
    }

    /// Try to pop the next buffered preview event without waiting.
    pub fn try_next(&self) -> Option<PreviewEvent> {
        self.inner.try_next()
    }

    /// Number of buffered preview events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }
}

impl std::fmt::Debug for PreviewEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewEventStream")
            .field("buffered_count", &self.buffered_count())
            .finish_non_exhaustive()
    }
}

/// Bounded async stream of typed [`DetailedRecordingEvent`] values.
pub struct DetailedRecordingEventStream {
    inner: BoundedAsyncStream<DetailedRecordingEvent>,
    _observer: DetailedRecordingObserver,
}

impl DetailedRecordingEventStream {
    /// Register a bounded async stream backed by [`ScreenRecorder::observe_detailed`].
    #[must_use]
    pub fn observe(recorder: &ScreenRecorder, capacity: usize) -> Self {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let observer = recorder.observe_detailed(move |event| {
            catch_user_panic(
                "replaykit::async_api::detailed_recording_event_stream",
                || {
                    sender.push(event);
                },
            );
        });

        Self {
            inner: stream,
            _observer: observer,
        }
    }

    /// Await the next buffered detailed recording event.
    pub const fn next(&self) -> NextItem<'_, DetailedRecordingEvent> {
        self.inner.next()
    }

    /// Try to pop the next buffered detailed recording event without waiting.
    pub fn try_next(&self) -> Option<DetailedRecordingEvent> {
        self.inner.try_next()
    }

    /// Number of buffered detailed recording events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }
}

impl std::fmt::Debug for DetailedRecordingEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetailedRecordingEventStream")
            .field("buffered_count", &self.buffered_count())
            .finish_non_exhaustive()
    }
}

/// Bounded async stream of [`CaptureEvent`] values.
///
/// Construction reuses the crate's existing blocking `start_capture` bridge for
/// setup; once capture is active, sample events are delivered asynchronously.
pub struct SampleBufferCaptureEventStream {
    inner: BoundedAsyncStream<CaptureEvent>,
    session: Option<SampleBufferCaptureSession>,
}

impl SampleBufferCaptureEventStream {
    /// Whether sample-buffer capture is available on this platform.
    pub fn is_supported_on_current_platform() -> bool {
        SampleBufferCaptureSession::is_supported_on_current_platform()
    }

    /// Start capture and expose sample-buffer callbacks as a bounded async stream.
    pub fn start(recorder: &ScreenRecorder, capacity: usize) -> Result<Self, ReplayKitError> {
        let (stream, sender) = BoundedAsyncStream::new(capacity);
        let session = recorder.start_capture(move |event| {
            catch_user_panic(
                "replaykit::async_api::sample_buffer_capture_event_stream",
                || {
                    sender.push(event);
                },
            );
        })?;

        Ok(Self {
            inner: stream,
            session: Some(session),
        })
    }

    /// Stop capture and release the underlying delegate bridge.
    pub fn stop(mut self) -> Result<(), ReplayKitError> {
        self.session
            .take()
            .map_or(Ok(()), SampleBufferCaptureSession::stop)
    }

    /// Await the next buffered capture event.
    pub const fn next(&self) -> NextItem<'_, CaptureEvent> {
        self.inner.next()
    }

    /// Try to pop the next buffered capture event without waiting.
    pub fn try_next(&self) -> Option<CaptureEvent> {
        self.inner.try_next()
    }

    /// Number of buffered capture events.
    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }
}

impl std::fmt::Debug for SampleBufferCaptureEventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleBufferCaptureEventStream")
            .field("buffered_count", &self.buffered_count())
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Namespaces
// ============================================================================

/// Async accessor for `RPBroadcastActivityController`.
pub struct AsyncBroadcastActivityControllerHandle;

impl AsyncBroadcastActivityControllerHandle {
    /// Present the macOS broadcast picker and await the selected [`BroadcastController`].
    #[must_use = "futures do nothing unless awaited"]
    pub fn show(
        origin: (f64, f64),
        preferred_extension: Option<&str>,
    ) -> AsyncShowBroadcastActivity {
        let preferred_extension = match preferred_extension {
            Some(value) => match cstring_from_str(value, "preferred broadcast extension") {
                Ok(value) => Some(value),
                Err(error) => return AsyncShowBroadcastActivity::ready(Err(error)),
            },
            None => None,
        };
        let extension_ptr = preferred_extension
            .as_ref()
            .map_or(ptr::null(), |value| value.as_ptr());

        let (future, ctx) = AsyncCompletion::<BroadcastController>::create();
        // SAFETY: `ctx` comes from `AsyncCompletion::create()` and the Swift bridge
        // calls `broadcast_activity_callback` exactly once for this picker flow.
        unsafe {
            ffi::rk_broadcast_activity_controller_show(
                origin.0,
                origin.1,
                ptr::null_mut(),
                extension_ptr,
                ctx,
                broadcast_activity_callback,
            );
        }

        AsyncShowBroadcastActivity {
            inner: AsyncShowBroadcastActivityInner::Pending(future),
        }
    }
}

/// Async operations for `RPScreenRecorder`.
pub struct AsyncScreenRecorder;

impl AsyncScreenRecorder {
    /// Start recording asynchronously.
    #[must_use = "futures do nothing unless awaited"]
    pub fn start_recording(recorder: &ScreenRecorder) -> AsyncStartRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: `recorder.as_ptr()` is a valid `RPScreenRecorder` pointer and `ctx`
        // comes from `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_start_recording_async(
                recorder.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncStartRecording { inner: future }
    }

    /// Stop recording asynchronously, optionally returning a preview controller.
    #[must_use = "futures do nothing unless awaited"]
    pub fn stop_recording(recorder: &ScreenRecorder) -> AsyncStopRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: `recorder.as_ptr()` is a valid `RPScreenRecorder` pointer and `ctx`
        // comes from `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_stop_recording_async(
                recorder.as_ptr(),
                preview_callback,
                ctx,
            );
        }
        AsyncStopRecording { inner: future }
    }

    /// Stop recording asynchronously and write to `output_path`.
    #[must_use = "futures do nothing unless awaited"]
    pub fn stop_recording_with_output<P: AsRef<Path>>(
        recorder: &ScreenRecorder,
        output_path: P,
    ) -> AsyncStopRecordingWithOutput {
        let output_path = match path_cstring(output_path.as_ref(), "recording output path") {
            Ok(output_path) => output_path,
            Err(error) => {
                return AsyncStopRecordingWithOutput {
                    inner: AsyncStopRecordingWithOutputInner::Ready(future::ready(Err(error))),
                }
            }
        };
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: `recorder.as_ptr()` is a valid `RPScreenRecorder` pointer,
        // `output_path` is a valid C string, and `ctx` comes from
        // `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_stop_recording_with_output_async(
                recorder.as_ptr(),
                output_path.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncStopRecordingWithOutput {
            inner: AsyncStopRecordingWithOutputInner::Pending(future),
        }
    }

    /// Discard the current recording asynchronously.
    #[must_use = "futures do nothing unless awaited"]
    pub fn discard_recording(recorder: &ScreenRecorder) -> AsyncDiscardRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: `recorder.as_ptr()` is a valid `RPScreenRecorder` pointer and `ctx`
        // comes from `AsyncCompletion::create()`.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_discard_recording_async(
                recorder.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncDiscardRecording { inner: future }
    }

    /// Observe typed recorder delegate events as a bounded async stream.
    #[must_use]
    pub fn detailed_events(
        recorder: &ScreenRecorder,
        capacity: usize,
    ) -> DetailedRecordingEventStream {
        DetailedRecordingEventStream::observe(recorder, capacity)
    }

    /// Start sample-buffer capture and expose the captured samples as a bounded
    /// async stream.
    pub fn capture_events(
        recorder: &ScreenRecorder,
        capacity: usize,
    ) -> Result<SampleBufferCaptureEventStream, ReplayKitError> {
        SampleBufferCaptureEventStream::start(recorder, capacity)
    }
}

#[cfg(test)]
mod tests {
    use core::ffi::c_char;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr;

    use doom_fish_utils::completion::AsyncCompletion;

    use super::{
        preview_callback, void_callback, AsyncBroadcastActivityControllerHandle,
        AsyncDiscardRecording, AsyncScreenRecorder, AsyncShowBroadcastActivity,
        AsyncStartRecording, AsyncStopRecording, AsyncStopRecordingWithOutput,
        SampleBufferCaptureEventStream,
    };
    use crate::ffi::status;
    use crate::preview_view::PreviewViewControllerHandle;
    use crate::{
        sample_buffer_delegate::SampleBufferCaptureSession, RecordingErrorCode, ReplayKitError,
        ScreenRecorder,
    };

    extern "C" {
        fn strdup(value: *const c_char) -> *mut c_char;
    }

    const NOT_RECORDING: &std::ffi::CStr = c"{\"kind\":\"framework\",\"domain\":\"com.apple.ReplayKit.RPRecordingErrorDomain\",\"code\":-5829,\"localizedDescription\":\"not recording\"}";

    const fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn recording_futures_are_send_and_sync() {
        assert_send_sync::<AsyncStartRecording>();
        assert_send_sync::<AsyncStopRecording>();
        assert_send_sync::<AsyncStopRecordingWithOutput>();
        assert_send_sync::<AsyncDiscardRecording>();
        assert_send_sync::<AsyncShowBroadcastActivity>();
    }

    #[test]
    fn async_failures_map_like_the_sync_bridge() {
        let (future, ctx) = AsyncCompletion::<Result<(), ReplayKitError>>::create();
        void_callback(
            ptr::null(),
            status::FRAMEWORK_ERROR,
            unsafe { strdup(NOT_RECORDING.as_ptr()) },
            ctx,
        );
        let Err(ReplayKitError::Framework(error)) =
            pollster::block_on(AsyncStartRecording { inner: future })
        else {
            panic!("expected a framework error");
        };
        assert_eq!(
            error.recording_code(),
            Some(RecordingErrorCode::AttemptToStopNonRecording)
        );
        assert_eq!(error.localized_description, "not recording");

        let (future, ctx) = AsyncCompletion::<Result<(), ReplayKitError>>::create();
        void_callback(
            ptr::null(),
            status::INVALID_ARGUMENT,
            unsafe { strdup(c"missing file-system path".as_ptr()) },
            ctx,
        );
        assert_eq!(
            pollster::block_on(AsyncDiscardRecording { inner: future }),
            Err(ReplayKitError::InvalidArgument(
                "missing file-system path".into()
            ))
        );

        let (future, ctx) = AsyncCompletion::<Result<(), ReplayKitError>>::create();
        void_callback(ptr::null(), status::OK, ptr::null_mut(), ctx);
        assert_eq!(pollster::block_on(AsyncStartRecording { inner: future }), Ok(()));
    }

    #[test]
    fn preview_completions_carry_typed_errors() {
        let (future, ctx) =
            AsyncCompletion::<Result<Option<PreviewViewControllerHandle>, ReplayKitError>>::create();
        preview_callback(
            ptr::null(),
            status::FRAMEWORK_ERROR,
            unsafe { strdup(NOT_RECORDING.as_ptr()) },
            ctx,
        );
        assert!(matches!(
            pollster::block_on(AsyncStopRecording { inner: future }),
            Err(ReplayKitError::Framework(_))
        ));

        let (future, ctx) =
            AsyncCompletion::<Result<Option<PreviewViewControllerHandle>, ReplayKitError>>::create();
        preview_callback(ptr::null(), status::OK, ptr::null_mut(), ctx);
        assert!(matches!(
            pollster::block_on(AsyncStopRecording { inner: future }),
            Ok(None)
        ));
    }

    #[test]
    fn stop_recording_with_output_rejects_invalid_paths() {
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let nul = pollster::block_on(AsyncScreenRecorder::stop_recording_with_output(
            &recorder,
            "target/bad\0path.mov",
        ));
        assert!(matches!(nul, Err(ReplayKitError::InvalidArgument(_))));

        let non_utf8 = Path::new(OsStr::from_bytes(b"target/\xff.mov"));
        let result = pollster::block_on(AsyncScreenRecorder::stop_recording_with_output(
            &recorder, non_utf8,
        ));
        assert!(matches!(result, Err(ReplayKitError::InvalidArgument(_))));
    }

    #[test]
    fn async_broadcast_activity_show_rejects_nul_extension_identifier() {
        let result = pollster::block_on(AsyncBroadcastActivityControllerHandle::show(
            (0.0, 0.0),
            Some("bad\0extension"),
        ));
        assert!(matches!(result, Err(ReplayKitError::InvalidArgument(_))));
    }

    #[test]
    fn capture_event_stream_support_matches_sync_bridge() {
        assert_eq!(
            SampleBufferCaptureEventStream::is_supported_on_current_platform(),
            SampleBufferCaptureSession::is_supported_on_current_platform(),
        );
    }

    #[test]
    fn detailed_event_stream_can_be_created() {
        let _lock = crate::private::recorder_test_lock();
        let Some(recorder) = ScreenRecorder::shared() else {
            return;
        };

        let stream = AsyncScreenRecorder::detailed_events(&recorder, 4);
        assert!(stream.buffered_count() <= 1);
        let _ = stream.try_next();
    }
}
