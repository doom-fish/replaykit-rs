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
    if error.is_null() {
        unsafe { AsyncCompletion::<()>::complete_ok(user_data, ()) };
    } else {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<()>::complete_err(user_data, msg) };
    }
}

// ============================================================================
// Callback function for preview controller results
// ============================================================================

extern "C" fn preview_callback(
    result: *const c_void,
    error: *const i8,
    user_data: *mut c_void,
) {
    if error.is_null() {
        if result.is_null() {
            unsafe { AsyncCompletion::complete_ok(user_data, None::<PreviewViewController>) };
        } else {
            let preview = unsafe { PreviewViewController::from_ptr(result.cast_mut()) };
            unsafe { AsyncCompletion::complete_ok(user_data, Some(preview)) };
        }
    } else {
        let msg = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<Option<PreviewViewController>>::complete_err(user_data, msg) };
    }
}

// ============================================================================
// AsyncStartRecording Future
// ============================================================================

/// Future for async start recording operation
pub struct AsyncStartRecording {
    inner: AsyncCompletionFuture<()>,
}

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
