use core::ffi::{c_char, c_void};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use doom_fish_utils::callback_context::CallbackContext;
use serde::Deserialize;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::preview_view::PreviewViewControllerHandle;
use crate::private::{error_from_status, parse_json_ptr, path_cstring, result_from_status};

type RecordingHandler = Box<dyn Fn(RecordingEvent) + Send + Sync>;
type DetailedRecordingHandler = Box<dyn Fn(DetailedRecordingEvent) + Send + Sync>;

/// Events forwarded by the lightweight `RPScreenRecorderDelegate` bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingEvent {
    /// Recording stopped (possibly with an error).
    DidStopRecording { error: Option<ReplayKitError> },
    /// The recorder's availability changed.
    AvailabilityChanged { is_available: bool },
}

/// Detailed events forwarded by `RPScreenRecorderDelegate`.
#[derive(Debug)]
pub enum DetailedRecordingEvent {
    /// Recording stopped and optionally produced a preview controller and/or error.
    DidStopRecording {
        /// Preview controller returned by `ReplayKit` when available.
        preview_view_controller: Option<PreviewViewControllerHandle>,
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
}

impl From<ScreenRecorderStatePayload> for ScreenRecorderState {
    fn from(value: ScreenRecorderStatePayload) -> Self {
        Self {
            is_available: value.is_available,
            is_recording: value.is_recording,
            is_microphone_enabled: value.is_microphone_enabled,
            is_camera_enabled: value.is_camera_enabled,
            camera_position: CameraPosition::from_raw(value.camera_position),
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

    pub(crate) fn retained(&self) -> Self {
        Self {
            ptr: unsafe { ffi::rk_object_retain(self.ptr) },
        }
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
    pub fn set_camera_position(&self, position: CameraPosition) -> Result<(), ReplayKitError> {
        if let CameraPosition::Unknown(raw) = position {
            return Err(ReplayKitError::InvalidArgument(format!(
                "camera position {raw} is neither front (1) nor back (2)"
            )));
        }
        unsafe { ffi::rk_screen_recorder_set_camera_position(self.ptr, position.as_raw()) };
        Ok(())
    }

    /// Returns the current camera preview view when camera capture is enabled.
    pub fn camera_preview_view(&self) -> Result<Option<CameraPreviewView>, ReplayKitError> {
        let mut view: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_camera_preview_view(self.ptr, &raw mut view, &raw mut err)
        };
        result_from_status(rc, err)?;
        Ok((!view.is_null()).then_some(CameraPreviewView {
            ptr: view,
            _main_thread_only: PhantomData,
        }))
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
    ) -> Result<Option<PreviewViewControllerHandle>, ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let mut preview_ptr: *mut c_void = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_stop_recording_with_preview(
                self.ptr,
                &raw mut preview_ptr,
                &raw mut err,
            )
        };
        let preview = unsafe { PreviewViewControllerHandle::from_raw(preview_ptr) };
        if rc == crate::ffi::status::OK {
            Ok(preview)
        } else {
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
        if !(duration_seconds.is_finite() && duration_seconds > 0.0) {
            return Err(ReplayKitError::InvalidArgument(format!(
                "clip duration must be a positive number of seconds, got {duration_seconds}"
            )));
        }
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
        F: Fn(RecordingEvent) + Send + Sync + 'static,
    {
        let handler: RecordingHandler = Box::new(handler);
        let context = CallbackContext::new(handler);
        let token = unsafe {
            ffi::rk_screen_recorder_add_summary_observer(
                self.ptr,
                summary_trampoline,
                context.as_ptr(),
                CallbackContext::<RecordingHandler>::RETAIN,
                CallbackContext::<RecordingHandler>::RELEASE,
            )
        };
        RecordingObserver { token, context }
    }

    /// Registers a delegate callback that receives typed [`DetailedRecordingEvent`] values.
    pub fn observe_detailed<F>(&self, handler: F) -> DetailedRecordingObserver
    where
        F: Fn(DetailedRecordingEvent) + Send + Sync + 'static,
    {
        let handler: DetailedRecordingHandler = Box::new(handler);
        let context = CallbackContext::new(handler);
        let token = unsafe {
            ffi::rk_screen_recorder_add_detailed_observer(
                self.ptr,
                detailed_trampoline,
                context.as_ptr(),
                CallbackContext::<DetailedRecordingHandler>::RETAIN,
                CallbackContext::<DetailedRecordingHandler>::RELEASE,
            )
        };
        DetailedRecordingObserver { token, context }
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

unsafe fn recording_error(error_json: *mut c_char) -> Option<ReplayKitError> {
    if error_json.is_null() {
        return None;
    }
    let message = unsafe { crate::private::take_string(error_json) }
        .unwrap_or_else(|| "recording delegate error".into());
    Some(crate::error::from_message(&message))
}

fn unknown_event_kind(event_kind: i32) -> ReplayKitError {
    ReplayKitError::Unknown(format!(
        "unknown recording delegate event kind: {event_kind}"
    ))
}

unsafe extern "C" fn summary_trampoline(
    context: *mut c_void,
    event_kind: i32,
    is_available: bool,
    error_json: *mut c_char,
) {
    let error = unsafe { recording_error(error_json) };
    let event = match event_kind {
        1 => RecordingEvent::AvailabilityChanged { is_available },
        2 => RecordingEvent::DidStopRecording { error },
        other => RecordingEvent::DidStopRecording {
            error: Some(unknown_event_kind(other)),
        },
    };
    unsafe {
        CallbackContext::<RecordingHandler>::with(
            context,
            "replaykit::screen_recorder::summary_trampoline",
            |handler| handler(event),
        )
    };
}

unsafe extern "C" fn detailed_trampoline(
    context: *mut c_void,
    event_kind: i32,
    is_available: bool,
    preview_controller_ptr: *mut c_void,
    error_json: *mut c_char,
) {
    let preview_view_controller =
        unsafe { PreviewViewControllerHandle::from_raw(preview_controller_ptr) };
    let error = unsafe { recording_error(error_json) };
    let event = match event_kind {
        1 => DetailedRecordingEvent::AvailabilityChanged { is_available },
        2 => DetailedRecordingEvent::DidStopRecording {
            preview_view_controller,
            error,
        },
        other => DetailedRecordingEvent::DidStopRecording {
            preview_view_controller,
            error: Some(unknown_event_kind(other)),
        },
    };
    unsafe {
        CallbackContext::<DetailedRecordingHandler>::with(
            context,
            "replaykit::screen_recorder::detailed_trampoline",
            |handler| handler(event),
        )
    };
}

/// Lightweight retained wrapper around the camera preview `NSView`.
pub struct CameraPreviewView {
    ptr: *mut c_void,
    _main_thread_only: PhantomData<Rc<()>>,
}

impl CameraPreviewView {
    /// Returns the Objective-C class name for the wrapped preview view.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { crate::private::take_string(ptr) }.unwrap_or_else(|| "NSView".into())
    }

    /// Returns whether the preview view is hidden.
    pub fn is_hidden(&self) -> Result<bool, ReplayKitError> {
        let mut hidden = false;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_ns_view_is_hidden(self.ptr, &raw mut hidden, &raw mut err) };
        result_from_status(rc, err).map(|()| hidden)
    }
}

impl Drop for CameraPreviewView {
    fn drop(&mut self) {
        unsafe { ffi::rk_object_release_on_main_thread(self.ptr) };
    }
}

impl std::fmt::Debug for CameraPreviewView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameraPreviewView")
            .field("class_name", &self.class_name())
            .finish()
    }
}

/// RAII guard returned by [`ScreenRecorder::observe`].
pub struct RecordingObserver {
    token: u64,
    context: CallbackContext<RecordingHandler>,
}

impl Drop for RecordingObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe { ffi::rk_screen_recorder_remove_observer(self.token) };
    }
}

impl std::fmt::Debug for RecordingObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordingObserver").finish_non_exhaustive()
    }
}

/// RAII guard returned by [`ScreenRecorder::observe_detailed`].
pub struct DetailedRecordingObserver {
    token: u64,
    context: CallbackContext<DetailedRecordingHandler>,
}

impl Drop for DetailedRecordingObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe { ffi::rk_screen_recorder_remove_observer(self.token) };
    }
}

impl std::fmt::Debug for DetailedRecordingObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetailedRecordingObserver")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use core::ffi::{c_char, c_void};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use super::{DetailedRecordingEvent, RecordingEvent, ScreenRecorder};
    use crate::error::{RecordingErrorCode, ReplayKitError, RP_RECORDING_ERROR_DOMAIN};
    use crate::private::recorder_test_lock;

    extern "C" {
        fn sel_registerName(name: *const c_char) -> *mut c_void;
        fn objc_getClass(name: *const c_char) -> *mut c_void;
        fn objc_msgSend();
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
        fn CFStringCreateWithCString(
            allocator: *const c_void,
            value: *const c_char,
            encoding: u32,
        ) -> *mut c_void;
        fn CFRelease(value: *const c_void);
    }

    fn recording_error(code: isize) -> *mut c_void {
        let domain = std::ffi::CString::new(RP_RECORDING_ERROR_DOMAIN).expect("domain");
        let domain =
            unsafe { CFStringCreateWithCString(std::ptr::null(), domain.as_ptr(), 0x0800_0100) };
        let send = unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(),
                unsafe extern "C" fn(
                    *mut c_void,
                    *mut c_void,
                    *mut c_void,
                    isize,
                    *mut c_void,
                ) -> *mut c_void,
            >(objc_msgSend)
        };
        let error = unsafe {
            send(
                objc_getClass(c"NSError".as_ptr()),
                sel_registerName(c"errorWithDomain:code:userInfo:".as_ptr()),
                domain,
                code,
                std::ptr::null_mut(),
            )
        };
        unsafe { CFRelease(domain) };
        error
    }

    fn with_autorelease_pool<R>(body: impl FnOnce() -> R) -> R {
        let pool = unsafe { objc_autoreleasePoolPush() };
        let result = body();
        unsafe { objc_autoreleasePoolPop(pool) };
        result
    }

    fn has_delegate(recorder: &ScreenRecorder) -> bool {
        with_autorelease_pool(|| !recorder_delegate(recorder).is_null())
    }

    fn recorder_delegate(recorder: &ScreenRecorder) -> *mut c_void {
        let send = unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(),
                unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
            >(objc_msgSend)
        };
        unsafe { send(recorder.as_ptr(), sel_registerName(c"delegate".as_ptr())) }
    }

    fn notify_availability_changed(recorder: &ScreenRecorder) {
        with_autorelease_pool(|| {
            let delegate = recorder_delegate(recorder);
            if delegate.is_null() {
                return;
            }
            let send = unsafe {
                std::mem::transmute::<
                    unsafe extern "C" fn(),
                    unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
                >(objc_msgSend)
            };
            unsafe {
                send(
                    delegate,
                    sel_registerName(c"screenRecorderDidChangeAvailability:".as_ptr()),
                    recorder.as_ptr(),
                );
            }
        });
    }

    fn notify_did_stop_recording(recorder: &ScreenRecorder, error: *mut c_void) {
        with_autorelease_pool(|| {
            let delegate = recorder_delegate(recorder);
            if delegate.is_null() {
                return;
            }
            let send = unsafe {
                std::mem::transmute::<
                    unsafe extern "C" fn(),
                    unsafe extern "C" fn(
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                        *mut c_void,
                    ),
                >(objc_msgSend)
            };
            unsafe {
                send(
                    delegate,
                    sel_registerName(
                        c"screenRecorder:didStopRecordingWithPreviewViewController:error:".as_ptr(),
                    ),
                    recorder.as_ptr(),
                    std::ptr::null_mut(),
                    error,
                );
            }
        });
    }

    fn counter() -> (Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let hits = Arc::new(AtomicUsize::new(0));
        (Arc::clone(&hits), hits)
    }

    #[test]
    fn observers_share_the_recorder_delegate_slot() {
        let _lock = recorder_test_lock();
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let (summary_hits, summary_sink) = counter();
        let (detailed_hits, detailed_sink) = counter();

        let summary = recorder.observe(move |event| {
            if matches!(event, RecordingEvent::AvailabilityChanged { .. }) {
                summary_sink.fetch_add(1, Ordering::SeqCst);
            }
        });
        let detailed = recorder.observe_detailed(move |event| {
            if matches!(event, DetailedRecordingEvent::AvailabilityChanged { .. }) {
                detailed_sink.fetch_add(1, Ordering::SeqCst);
            }
        });
        assert!(has_delegate(&recorder));
        let summary_before = summary_hits.load(Ordering::SeqCst);
        let detailed_before = detailed_hits.load(Ordering::SeqCst);

        notify_availability_changed(&recorder);
        assert_eq!(summary_hits.load(Ordering::SeqCst), summary_before + 1);
        assert_eq!(detailed_hits.load(Ordering::SeqCst), detailed_before + 1);

        drop(summary);
        assert!(has_delegate(&recorder));
        notify_availability_changed(&recorder);
        assert_eq!(summary_hits.load(Ordering::SeqCst), summary_before + 1);
        assert_eq!(detailed_hits.load(Ordering::SeqCst), detailed_before + 2);

        drop(detailed);
        assert!(!has_delegate(&recorder));
        notify_availability_changed(&recorder);
        assert_eq!(detailed_hits.load(Ordering::SeqCst), detailed_before + 2);
    }

    #[test]
    fn did_stop_recording_reaches_both_observer_kinds() {
        let _lock = recorder_test_lock();
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let summary_events = Arc::new(Mutex::new(Vec::new()));
        let detailed_events = Arc::new(Mutex::new(Vec::new()));

        let summary_sink = Arc::clone(&summary_events);
        let _summary = recorder.observe(move |event| {
            if let RecordingEvent::DidStopRecording { error } = event {
                summary_sink.lock().unwrap().push(error);
            }
        });
        let detailed_sink = Arc::clone(&detailed_events);
        let _detailed = recorder.observe_detailed(move |event| {
            if let DetailedRecordingEvent::DidStopRecording {
                preview_view_controller,
                error,
            } = event
            {
                detailed_sink
                    .lock()
                    .unwrap()
                    .push((preview_view_controller.is_some(), error));
            }
        });

        notify_did_stop_recording(&recorder, std::ptr::null_mut());

        assert_eq!(*summary_events.lock().unwrap(), vec![None]);
        assert_eq!(*detailed_events.lock().unwrap(), vec![(false, None)]);
    }

    #[test]
    fn did_stop_recording_errors_are_typed_for_both_observer_kinds() {
        let _lock = recorder_test_lock();
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let summary_errors = Arc::new(Mutex::new(Vec::new()));
        let detailed_errors = Arc::new(Mutex::new(Vec::new()));

        let summary_sink = Arc::clone(&summary_errors);
        let _summary = recorder.observe(move |event| {
            if let RecordingEvent::DidStopRecording { error } = event {
                summary_sink.lock().unwrap().push(error);
            }
        });
        let detailed_sink = Arc::clone(&detailed_errors);
        let _detailed = recorder.observe_detailed(move |event| {
            if let DetailedRecordingEvent::DidStopRecording { error, .. } = event {
                detailed_sink.lock().unwrap().push(error);
            }
        });

        with_autorelease_pool(|| notify_did_stop_recording(&recorder, recording_error(-5804)));

        let summary = std::mem::take(&mut *summary_errors.lock().unwrap());
        let detailed = std::mem::take(&mut *detailed_errors.lock().unwrap());
        assert_eq!(summary, detailed);
        assert_eq!(summary.len(), 1);
        let Some(ReplayKitError::Framework(error)) = &summary[0] else {
            panic!("expected a framework error, got {:?}", summary[0]);
        };
        assert_eq!(error.domain, RP_RECORDING_ERROR_DOMAIN);
        assert_eq!(error.recording_code(), Some(RecordingErrorCode::Failed));
    }

    #[test]
    fn dropping_an_observer_releases_its_handler() {
        let _lock = recorder_test_lock();
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let summary_token = Arc::new(());
        let detailed_token = Arc::new(());

        let held = Arc::clone(&summary_token);
        let summary = recorder.observe(move |_| {
            let _ = &held;
        });
        let held = Arc::clone(&detailed_token);
        let detailed = recorder.observe_detailed(move |_| {
            let _ = &held;
        });
        assert_eq!(Arc::strong_count(&summary_token), 2);
        assert_eq!(Arc::strong_count(&detailed_token), 2);

        drop(summary);
        drop(detailed);
        assert_eq!(Arc::strong_count(&summary_token), 1);
        assert_eq!(Arc::strong_count(&detailed_token), 1);
    }

    #[test]
    fn observers_can_be_dropped_while_events_are_in_flight() {
        let _lock = recorder_test_lock();
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        let stop = Arc::new(AtomicBool::new(false));
        let notifier_stop = Arc::clone(&stop);
        let notifier = thread::spawn(move || {
            let recorder = ScreenRecorder::shared().expect("shared recorder");
            while !notifier_stop.load(Ordering::SeqCst) {
                notify_availability_changed(&recorder);
            }
        });

        let tokens: Vec<Arc<AtomicUsize>> = (0..300)
            .map(|_| {
                let (hits, sink) = counter();
                let observer = recorder.observe_detailed(move |_| {
                    sink.fetch_add(1, Ordering::SeqCst);
                });
                thread::yield_now();
                drop(observer);
                hits
            })
            .collect();

        stop.store(true, Ordering::SeqCst);
        notifier.join().expect("notifier thread");
        assert!(!has_delegate(&recorder));
        assert!(tokens.iter().all(|hits| Arc::strong_count(hits) == 1));
    }
}
