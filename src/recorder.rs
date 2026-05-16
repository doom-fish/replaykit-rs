use core::ffi::{c_char, c_void};
use std::ptr;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::ffi::status;

// ── Event types ───────────────────────────────────────────────────────────────

/// Events forwarded by the `RPScreenRecorderDelegate`.
#[derive(Debug, Clone)]
pub enum RecordingEvent {
    /// Recording stopped (possibly with an error).
    DidStopRecording { error: Option<String> },
    /// The recorder's availability changed.
    AvailabilityChanged { is_available: bool },
    /// An unrecognised event payload.
    Unknown(String),
}

// ── ScreenRecorder ────────────────────────────────────────────────────────────

/// A safe wrapper around `RPScreenRecorder.shared()`.
///
/// Obtain via [`ScreenRecorder::shared`].
pub struct ScreenRecorder {
    ptr: *mut c_void,
}

// Safety: RPScreenRecorder is thread-safe for property reads; we never mutate
// the pointer itself after construction.
unsafe impl Send for ScreenRecorder {}
unsafe impl Sync for ScreenRecorder {}

impl ScreenRecorder {
    /// Returns the shared `RPScreenRecorder` instance.
    ///
    /// Returns `None` if `ReplayKit` is unavailable on this host.
    pub fn shared() -> Option<Self> {
        let ptr = unsafe { ffi::rk_screen_recorder_shared() };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    /// Whether `ReplayKit` is available on this device / OS version.
    pub fn is_available(&self) -> bool {
        unsafe { ffi::rk_screen_recorder_is_available(self.ptr) }
    }

    /// Whether a recording session is currently in progress.
    pub fn is_recording(&self) -> bool {
        unsafe { ffi::rk_screen_recorder_is_recording(self.ptr) }
    }

    /// Whether microphone recording is enabled.
    pub fn is_microphone_enabled(&self) -> bool {
        unsafe { ffi::rk_screen_recorder_is_microphone_enabled(self.ptr) }
    }

    /// Whether camera recording is enabled.
    pub fn is_camera_enabled(&self) -> bool {
        unsafe { ffi::rk_screen_recorder_is_camera_enabled(self.ptr) }
    }

    /// Starts a recording session.
    ///
    /// Blocks until the user grants/denies permission or an error occurs.
    pub fn start_recording(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_start_recording(self.ptr, &raw mut err) };
        if rc == status::OK {
            Ok(())
        } else {
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }

    /// Stops the active recording session.
    ///
    /// Blocks until the recording is fully flushed or an error occurs.
    pub fn stop_recording(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_stop_recording(self.ptr, &raw mut err) };
        if rc == status::OK {
            Ok(())
        } else {
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }

    /// Registers a delegate callback that receives [`RecordingEvent`]s.
    ///
    /// Returns a [`RecordingObserver`] RAII guard.  Drop it to unregister.
    pub fn observe<F>(&self, handler: F) -> RecordingObserver
    where
        F: Fn(RecordingEvent) + Send + 'static,
    {
        let boxed: Box<dyn Fn(RecordingEvent) + Send + 'static> = Box::new(handler);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();
        let holder_ptr = unsafe {
            ffi::rk_screen_recorder_set_delegate(self.ptr, delegate_trampoline, refcon)
        };
        RecordingObserver {
            recorder_ptr: self.ptr,
            holder_ptr,
            refcon,
        }
    }
}

impl Drop for ScreenRecorder {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_screen_recorder_release(self.ptr) };
        }
    }
}

// MARK: - delegate trampoline

unsafe extern "C" fn delegate_trampoline(refcon: *mut c_void, event_json: *const c_char) {
    // Safety: refcon is always a valid Box<Box<dyn Fn(…)>>.
    let handler = &*(refcon.cast::<Box<dyn Fn(RecordingEvent) + Send + 'static>>());
    let event = parse_event(event_json);
    handler(event);
}

fn parse_event(json_ptr: *const c_char) -> RecordingEvent {
    if json_ptr.is_null() {
        return RecordingEvent::Unknown("(null event)".into());
    }
    let json = unsafe { std::ffi::CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();

    // Minimal hand-rolled JSON peek — avoids pulling serde into hot path for
    // simple payloads.
    if json.contains("\"availabilityChanged\"") {
        let is_available = json.contains("\"isAvailable\":true");
        RecordingEvent::AvailabilityChanged { is_available }
    } else if json.contains("\"didStopRecording\"") {
        let error = if json.contains("\"error\":null") {
            None
        } else {
            Some(json)
        };
        RecordingEvent::DidStopRecording { error }
    } else {
        RecordingEvent::Unknown(json)
    }
}

// ── RecordingObserver ─────────────────────────────────────────────────────────

/// RAII guard returned by [`ScreenRecorder::observe`].
///
/// The delegate is automatically deregistered when this value is dropped.
pub struct RecordingObserver {
    recorder_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    refcon: *mut c_void,
}

// Safety: the underlying recorder is thread-safe; the refcon Box is Send.
unsafe impl Send for RecordingObserver {}
unsafe impl Sync for RecordingObserver {}

impl Drop for RecordingObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_screen_recorder_clear_delegate(self.recorder_ptr, self.holder_ptr);
            // Reclaim the Box we created in observe().
            drop(Box::from_raw(
                self.refcon
                    .cast::<Box<dyn Fn(RecordingEvent) + Send + 'static>>(),
            ));
        }
    }
}

impl std::fmt::Debug for RecordingObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordingObserver").finish_non_exhaustive()
    }
}
