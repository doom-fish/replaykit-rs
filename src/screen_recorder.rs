use core::ffi::{c_char, c_void};
use std::path::Path;
use std::ptr;

use serde::Deserialize;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::preview_view::PreviewViewController;
use crate::private::{error_from_status, parse_json_ptr, path_cstring, result_from_status};

/// Events forwarded by the lightweight `RPScreenRecorderDelegate` bridge.
#[derive(Debug, Clone)]
pub enum RecordingEvent {
    /// Recording stopped (possibly with an error).
    DidStopRecording { error: Option<String> },
    /// The recorder's availability changed.
    AvailabilityChanged { is_available: bool },
    /// An unrecognised event payload.
    Unknown(String),
}

/// Detailed events forwarded by `RPScreenRecorderDelegate`.
#[derive(Debug)]
pub enum DetailedRecordingEvent {
    /// Recording stopped and optionally produced a preview controller and/or error.
    DidStopRecording {
        /// Preview controller returned by `ReplayKit` when available.
        preview_view_controller: Option<PreviewViewController>,
        /// Framework error returned by `ReplayKit` when available.
        error: Option<ReplayKitError>,
    },
    /// The recorder's availability changed.
    AvailabilityChanged { is_available: bool },
}

/// `ReplayKit` camera positions exposed by `RPScreenRecorder`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPosition {
    /// Front-facing camera.
    Front,
    /// Back-facing camera.
    Back,
    /// Any future or unknown raw camera-position value.
    Unknown(i32),
}

impl CameraPosition {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Front,
            2 => Self::Back,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw `RPCameraPosition` integer.
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Front => 1,
            Self::Back => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
struct ScreenRecorderStatePayload {
    #[serde(rename = "isAvailable")]
    is_available: bool,
    #[serde(rename = "isRecording")]
    is_recording: bool,
    #[serde(rename = "isMicrophoneEnabled")]
    is_microphone_enabled: bool,
    #[serde(rename = "isCameraEnabled")]
    is_camera_enabled: bool,
    #[serde(rename = "cameraPosition")]
    camera_position: i32,
    #[serde(rename = "hasCameraPreviewView")]
    has_camera_preview_view: bool,
}

/// Snapshot of the current `RPScreenRecorder` state.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenRecorderState {
    /// Whether `ReplayKit` is currently available.
    pub is_available: bool,
    /// Whether recording is active.
    pub is_recording: bool,
    /// Whether microphone capture is enabled.
    pub is_microphone_enabled: bool,
    /// Whether camera capture is enabled.
    pub is_camera_enabled: bool,
    /// The currently selected camera position.
    pub camera_position: CameraPosition,
    /// Whether a camera preview view is currently available.
    pub has_camera_preview_view: bool,
}

impl From<ScreenRecorderStatePayload> for ScreenRecorderState {
    fn from(value: ScreenRecorderStatePayload) -> Self {
        Self {
            is_available: value.is_available,
            is_recording: value.is_recording,
            is_microphone_enabled: value.is_microphone_enabled,
            is_camera_enabled: value.is_camera_enabled,
            camera_position: CameraPosition::from_raw(value.camera_position),
            has_camera_preview_view: value.has_camera_preview_view,
        }
    }
}

/// Safe wrapper around `RPScreenRecorder.shared()`.
pub struct ScreenRecorder {
    ptr: *mut c_void,
}

unsafe impl Send for ScreenRecorder {}
unsafe impl Sync for ScreenRecorder {}

impl ScreenRecorder {
    /// Returns the shared `RPScreenRecorder` instance.
    pub fn shared() -> Option<Self> {
        let ptr = unsafe { ffi::rk_screen_recorder_shared() };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub(crate) const fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }

    /// Returns a structured snapshot of the current recorder state.
    pub fn state(&self) -> Result<ScreenRecorderState, ReplayKitError> {
        let ptr = unsafe { ffi::rk_screen_recorder_state_json(self.ptr) };
        unsafe { parse_json_ptr::<ScreenRecorderStatePayload>(ptr, "screen recorder state") }
            .map(Into::into)
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

    /// Enables or disables microphone recording.
    pub fn set_microphone_enabled(&self, enabled: bool) {
        unsafe { ffi::rk_screen_recorder_set_microphone_enabled(self.ptr, enabled) };
    }

    /// Whether camera recording is enabled.
    pub fn is_camera_enabled(&self) -> bool {
        unsafe { ffi::rk_screen_recorder_is_camera_enabled(self.ptr) }
    }

    /// Enables or disables camera capture.
    pub fn set_camera_enabled(&self, enabled: bool) {
        unsafe { ffi::rk_screen_recorder_set_camera_enabled(self.ptr, enabled) };
    }

    /// Returns the configured camera position.
    pub fn camera_position(&self) -> CameraPosition {
        CameraPosition::from_raw(unsafe { ffi::rk_screen_recorder_camera_position(self.ptr) })
    }

    /// Sets the active camera position.
    pub fn set_camera_position(&self, position: CameraPosition) {
        unsafe { ffi::rk_screen_recorder_set_camera_position(self.ptr, position.as_raw()) };
    }

    /// Returns the current camera preview view when camera capture is enabled.
    pub fn camera_preview_view(&self) -> Option<CameraPreviewView> {
        let ptr = unsafe { ffi::rk_screen_recorder_camera_preview_view(self.ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(CameraPreviewView { ptr })
        }
    }

    /// Starts a recording session.
    pub fn start_recording(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_start_recording(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Stops the active recording session and discards any returned preview controller.
    pub fn stop_recording(&self) -> Result<(), ReplayKitError> {
        self.stop_recording_with_preview().map(|_| ())
    }

    /// Stops the active recording session and returns the preview controller when `ReplayKit` supplies one.
    pub fn stop_recording_with_preview(
        &self,
    ) -> Result<Option<PreviewViewController>, ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let mut preview_ptr: *mut c_void = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_stop_recording_with_preview(
                self.ptr,
                &raw mut preview_ptr,
                &raw mut err,
            )
        };
        if rc == crate::ffi::status::OK {
            Ok((!preview_ptr.is_null())
                .then(|| unsafe { PreviewViewController::from_ptr(preview_ptr) }))
        } else {
            if !preview_ptr.is_null() {
                unsafe { ffi::rk_object_release(preview_ptr) };
            }
            Err(unsafe { error_from_status(rc, err) })
        }
    }

    /// Stops recording and writes the movie directly to the supplied output path.
    pub fn stop_recording_to_output<P: AsRef<Path>>(
        &self,
        output_path: P,
    ) -> Result<(), ReplayKitError> {
        let output_path = path_cstring(output_path.as_ref(), "recording output path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_stop_recording_with_output_url(
                self.ptr,
                output_path.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Discards the current recording after `ReplayKit` has finished stopping it.
    pub fn discard_recording(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_discard_recording(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Starts clip buffering on macOS 12+.
    pub fn start_clip_buffering(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_start_clip_buffering(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Stops clip buffering on macOS 12+.
    pub fn stop_clip_buffering(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_stop_clip_buffering(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Exports the newest buffered clip segment to the supplied output path on macOS 12+.
    pub fn export_clip_to_output<P: AsRef<Path>>(
        &self,
        output_path: P,
        duration_seconds: f64,
    ) -> Result<(), ReplayKitError> {
        let output_path = path_cstring(output_path.as_ref(), "clip output path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_export_clip_to_output_url(
                self.ptr,
                output_path.as_ptr(),
                duration_seconds,
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Registers a delegate callback that receives lightweight [`RecordingEvent`] values.
    pub fn observe<F>(&self, handler: F) -> RecordingObserver
    where
        F: Fn(RecordingEvent) + Send + 'static,
    {
        let boxed: Box<dyn Fn(RecordingEvent) + Send + 'static> = Box::new(handler);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();
        let holder_ptr =
            unsafe { ffi::rk_screen_recorder_set_delegate(self.ptr, delegate_trampoline, refcon) };
        RecordingObserver {
            recorder_ptr: self.ptr,
            holder_ptr,
            refcon,
        }
    }

    /// Registers a delegate callback that receives typed [`DetailedRecordingEvent`] values.
    pub fn observe_detailed<F>(&self, handler: F) -> DetailedRecordingObserver
    where
        F: Fn(DetailedRecordingEvent) + Send + 'static,
    {
        let boxed: Box<dyn Fn(DetailedRecordingEvent) + Send + 'static> = Box::new(handler);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();
        let holder_ptr = unsafe {
            ffi::rk_screen_recorder_set_detailed_delegate(
                self.ptr,
                detailed_delegate_trampoline,
                refcon,
            )
        };
        DetailedRecordingObserver {
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

impl std::fmt::Debug for ScreenRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenRecorder")
            .field("is_available", &self.is_available())
            .field("is_recording", &self.is_recording())
            .field("is_microphone_enabled", &self.is_microphone_enabled())
            .field("is_camera_enabled", &self.is_camera_enabled())
            .field("camera_position", &self.camera_position())
            .finish()
    }
}

fn parse_event(json_ptr: *const c_char) -> RecordingEvent {
    if json_ptr.is_null() {
        return RecordingEvent::Unknown("(null event)".into());
    }
    let json = unsafe { std::ffi::CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();

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

unsafe extern "C" fn delegate_trampoline(refcon: *mut c_void, event_json: *const c_char) {
    let handler = &*(refcon.cast::<Box<dyn Fn(RecordingEvent) + Send + 'static>>());
    handler(parse_event(event_json));
}

unsafe extern "C" fn detailed_delegate_trampoline(
    refcon: *mut c_void,
    event_kind: i32,
    is_available: bool,
    preview_controller_ptr: *mut c_void,
    error_json: *mut c_char,
) {
    let handler = &*(refcon.cast::<Box<dyn Fn(DetailedRecordingEvent) + Send + 'static>>());
    let preview_view_controller = (!preview_controller_ptr.is_null())
        .then(|| PreviewViewController::from_ptr(preview_controller_ptr));
    let error = if error_json.is_null() {
        None
    } else {
        let message = crate::private::take_string(error_json)
            .unwrap_or_else(|| "recording delegate error".into());
        Some(crate::error::from_message(&message))
    };
    let event = match event_kind {
        1 => DetailedRecordingEvent::AvailabilityChanged { is_available },
        2 => DetailedRecordingEvent::DidStopRecording {
            preview_view_controller,
            error,
        },
        _ => DetailedRecordingEvent::DidStopRecording {
            preview_view_controller,
            error: Some(ReplayKitError::Unknown(format!(
                "unknown recording delegate event kind: {event_kind}"
            ))),
        },
    };
    handler(event);
}

/// Lightweight retained wrapper around the camera preview `NSView`.
pub struct CameraPreviewView {
    ptr: *mut c_void,
}

unsafe impl Send for CameraPreviewView {}
unsafe impl Sync for CameraPreviewView {}

impl CameraPreviewView {
    /// Returns the Objective-C class name for the wrapped preview view.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { crate::private::take_string(ptr) }.unwrap_or_else(|| "NSView".into())
    }

    /// Returns whether the preview view is hidden.
    pub fn is_hidden(&self) -> bool {
        unsafe { ffi::rk_ns_view_is_hidden(self.ptr) }
    }
}

impl Drop for CameraPreviewView {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_object_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for CameraPreviewView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameraPreviewView")
            .field("class_name", &self.class_name())
            .field("is_hidden", &self.is_hidden())
            .finish()
    }
}

/// RAII guard returned by [`ScreenRecorder::observe`].
pub struct RecordingObserver {
    recorder_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    refcon: *mut c_void,
}

unsafe impl Send for RecordingObserver {}
unsafe impl Sync for RecordingObserver {}

impl Drop for RecordingObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_screen_recorder_clear_delegate(self.recorder_ptr, self.holder_ptr);
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

/// RAII guard returned by [`ScreenRecorder::observe_detailed`].
pub struct DetailedRecordingObserver {
    recorder_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    refcon: *mut c_void,
}

unsafe impl Send for DetailedRecordingObserver {}
unsafe impl Sync for DetailedRecordingObserver {}

impl Drop for DetailedRecordingObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_screen_recorder_clear_detailed_delegate(self.recorder_ptr, self.holder_ptr);
            drop(Box::from_raw(
                self.refcon
                    .cast::<Box<dyn Fn(DetailedRecordingEvent) + Send + 'static>>(),
            ));
        }
    }
}

impl std::fmt::Debug for DetailedRecordingObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetailedRecordingObserver")
            .finish_non_exhaustive()
    }
}
