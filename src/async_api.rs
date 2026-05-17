//! Async API for `ReplayKit`
//!
//! This module provides async versions of `RPScreenRecorder` operations when the `async` feature is enabled.
//! The async API is **executor-agnostic** and works with any async runtime (Tokio, async-std, smol, etc.).
//!
//! ## Available Futures
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncStartRecording`] | Start recording asynchronously |
//! | [`AsyncStopRecording`] | Stop recording asynchronously |
//! | [`AsyncStopRecordingWithOutput`] | Stop recording with file output |
//! | [`AsyncStartCapture`] | Start sample buffer capture |
//! | [`AsyncStopCapture`] | Stop sample buffer capture |
//! | [`AsyncDiscardRecording`] | Discard current recording |
//!
//! ## Runtime Agnostic Design
//!
//! This async API uses only `std` types and works with **any** async runtime:
//! - Uses callback-based Swift FFI for true async operations
//! - Uses `std::sync::{Arc, Mutex}` for synchronization
//! - Uses `std::task::{Poll, Waker}` for async primitives
//! - Uses `std::future::Future` trait
//!
//! ## Examples
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use replaykit::prelude::*;
//! use replaykit::async_api::AsyncScreenRecorder;
//!
//! let recorder = ScreenRecorder::shared().expect("ReplayKit unavailable");
//! AsyncScreenRecorder::start_recording(&recorder).await?;
//! println!("Recording started");
//! # Ok(())
//! # }
//! ```

use crate::error::ReplayKitError;
use crate::preview_view::PreviewViewController;
use crate::screen_recorder::ScreenRecorder;
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::panic_safe::catch_user_panic;
use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// ============================================================================
// Callback function for void completions (no result)
// ============================================================================

extern "C" fn void_callback(
    _result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    catch_user_panic("replaykit::async_api::void_callback", || {
        if error.is_null() {
            // SAFETY: user_data is a valid pointer obtained from AsyncCompletion::create() and has never been
            // completed. This is guaranteed by the Swift bridge which only calls this once per async operation.
            unsafe { AsyncCompletion::<()>::complete_ok(user_data, ()) };
        } else {
            // SAFETY: error is a valid C string pointer from the Swift bridge. error_from_cstr safely handles
            // invalid UTF-8 and null checks.
            let msg = unsafe { error_from_cstr(error) };
            // SAFETY: user_data is a valid pointer from AsyncCompletion::create(), guaranteed called once.
            unsafe { AsyncCompletion::<()>::complete_err(user_data, msg) };
        }
    });
}

// ============================================================================
// Callback function for preview controller results
// ============================================================================

extern "C" fn preview_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    catch_user_panic("replaykit::async_api::preview_callback", || {
        if error.is_null() {
            if result.is_null() {
                // SAFETY: user_data is a valid pointer from AsyncCompletion::create(), guaranteed called once.
                unsafe { AsyncCompletion::complete_ok(user_data, None::<PreviewViewController>) };
            } else {
                // SAFETY: result is a valid RPPreviewViewController pointer from the Swift bridge. from_ptr
                // wraps it in a reference-counted handle. cast_mut is safe because we own the pointer from FFI.
                let preview = unsafe { PreviewViewController::from_ptr(result.cast_mut()) };
                // SAFETY: user_data is a valid pointer from AsyncCompletion::create(), guaranteed called once.
                unsafe { AsyncCompletion::complete_ok(user_data, Some(preview)) };
            }
        } else {
            // SAFETY: error is a valid C string pointer from the Swift bridge.
            let msg = unsafe { error_from_cstr(error) };
            // SAFETY: user_data is a valid pointer from AsyncCompletion::create(), guaranteed called once.
            unsafe { AsyncCompletion::<Option<PreviewViewController>>::complete_err(user_data, msg) };
        }
    });
}

// ============================================================================
// AsyncStartRecording Future
// ============================================================================

/// Future for async start recording operation
pub struct AsyncStartRecording {
    inner: AsyncCompletionFuture<()>,
}

// SAFETY: AsyncStartRecording is Send + Sync because:
// - AsyncCompletionFuture<()> is Send + Sync for Send payloads
// - The unit type () is Send + Sync
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncStartRecording {}
unsafe impl Sync for AsyncStartRecording {}

impl Future for AsyncStartRecording {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncStopRecording Future
// ============================================================================

/// Future for async stop recording operation (optionally returns preview controller)
pub struct AsyncStopRecording {
    inner: AsyncCompletionFuture<Option<PreviewViewController>>,
}

// SAFETY: AsyncStopRecording is Send + Sync because:
// - AsyncCompletionFuture<Option<PreviewViewController>> is Send + Sync
// - PreviewViewController wraps an Objective-C object reference, which is thread-safe
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncStopRecording {}
unsafe impl Sync for AsyncStopRecording {}

impl Future for AsyncStopRecording {
    type Output = Result<Option<PreviewViewController>, ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncStopRecordingWithOutput Future
// ============================================================================

/// Future for async stop recording with file output
pub struct AsyncStopRecordingWithOutput {
    inner: AsyncCompletionFuture<()>,
}

// SAFETY: AsyncStopRecordingWithOutput is Send + Sync because:
// - AsyncCompletionFuture<()> is Send + Sync for Send payloads
// - The unit type () is Send + Sync
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncStopRecordingWithOutput {}
unsafe impl Sync for AsyncStopRecordingWithOutput {}

impl Future for AsyncStopRecordingWithOutput {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncStartCapture Future
// ============================================================================

/// Future for async start capture operation
pub struct AsyncStartCapture {
    inner: AsyncCompletionFuture<()>,
}

// SAFETY: AsyncStartCapture is Send + Sync because:
// - AsyncCompletionFuture<()> is Send + Sync for Send payloads
// - The unit type () is Send + Sync
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncStartCapture {}
unsafe impl Sync for AsyncStartCapture {}

impl Future for AsyncStartCapture {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncStopCapture Future
// ============================================================================

/// Future for async stop capture operation
pub struct AsyncStopCapture {
    inner: AsyncCompletionFuture<()>,
}

// SAFETY: AsyncStopCapture is Send + Sync because:
// - AsyncCompletionFuture<()> is Send + Sync for Send payloads
// - The unit type () is Send + Sync
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncStopCapture {}
unsafe impl Sync for AsyncStopCapture {}

impl Future for AsyncStopCapture {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncDiscardRecording Future
// ============================================================================

/// Future for async discard recording operation
pub struct AsyncDiscardRecording {
    inner: AsyncCompletionFuture<()>,
}

// SAFETY: AsyncDiscardRecording is Send + Sync because:
// - AsyncCompletionFuture<()> is Send + Sync for Send payloads
// - The unit type () is Send + Sync
// - The Swift bridge guarantees thread-safe callback delivery
unsafe impl Send for AsyncDiscardRecording {}
unsafe impl Sync for AsyncDiscardRecording {}

impl Future for AsyncDiscardRecording {
    type Output = Result<(), ReplayKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|r| r.map_err(ReplayKitError::Unknown))
    }
}

// ============================================================================
// AsyncScreenRecorder - async operations on ScreenRecorder
// ============================================================================

/// Async operations for `RPScreenRecorder`
pub struct AsyncScreenRecorder;

impl AsyncScreenRecorder {
    /// Starts recording asynchronously.
    pub fn start_recording(recorder: &ScreenRecorder) -> AsyncStartRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: recorder.as_ptr() returns a valid RPScreenRecorder pointer. ctx is a valid completion context
        // from AsyncCompletion::create(). void_callback is a valid extern "C" function. The Swift bridge guarantees
        // it will call the callback exactly once.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_start_recording_async(
                recorder.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncStartRecording { inner: future }
    }

    /// Stops recording asynchronously, optionally returning a preview controller.
    pub fn stop_recording(recorder: &ScreenRecorder) -> AsyncStopRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: recorder.as_ptr() returns a valid RPScreenRecorder pointer. ctx is a valid completion context
        // from AsyncCompletion::create(). preview_callback is a valid extern "C" function that handles both
        // null and non-null result pointers. The Swift bridge guarantees it will call the callback exactly once.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_stop_recording_async(
                recorder.as_ptr(),
                preview_callback,
                ctx,
            );
        }
        AsyncStopRecording { inner: future }
    }

    /// Stops recording asynchronously and writes to the specified output file.
    pub fn stop_recording_with_output(
        recorder: &ScreenRecorder,
        output_path: &std::ffi::CStr,
    ) -> AsyncStopRecordingWithOutput {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: recorder.as_ptr() returns a valid RPScreenRecorder pointer. output_path.as_ptr() is a valid
        // C string pointer (CStr guarantees valid null-terminated UTF-8). ctx is a valid completion context from
        // AsyncCompletion::create(). void_callback is a valid extern "C" function. The Swift bridge guarantees
        // it will call the callback exactly once.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_stop_recording_with_output_async(
                recorder.as_ptr(),
                output_path.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncStopRecordingWithOutput { inner: future }
    }

    /// Discards the current recording asynchronously.
    pub fn discard_recording(recorder: &ScreenRecorder) -> AsyncDiscardRecording {
        let (future, ctx) = AsyncCompletion::create();
        // SAFETY: recorder.as_ptr() returns a valid RPScreenRecorder pointer. ctx is a valid completion context
        // from AsyncCompletion::create(). void_callback is a valid extern "C" function. The Swift bridge guarantees
        // it will call the callback exactly once.
        unsafe {
            crate::ffi::async_api::rk_screen_recorder_discard_recording_async(
                recorder.as_ptr(),
                void_callback,
                ctx,
            );
        }
        AsyncDiscardRecording { inner: future }
    }
}

// Note: startCapture and stopCapture are more complex as startCapture involves
// a streaming callback in addition to the completion. These are deferred to Tier 2
// Stream API implementation for proper support.
