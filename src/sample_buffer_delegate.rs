use core::ffi::{c_char, c_void};
use std::ptr;

use apple_cf::cm::CMSampleBuffer;
use doom_fish_utils::callback_context::CallbackContext;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{result_from_status, take_string};
use crate::screen_recorder::ScreenRecorder;

const SAMPLE_EVENT: i32 = 1;
const ERROR_EVENT: i32 = 2;

type CaptureHandler = Box<dyn SampleBufferDelegate>;

/// `ReplayKit` sample-buffer kinds emitted by `startCaptureWithHandler`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleBufferType {
    /// Video sample.
    Video,
    /// Application audio sample.
    AudioApp,
    /// Microphone audio sample.
    AudioMic,
    /// Any future or unknown raw buffer type.
    Unknown(i32),
}

impl SampleBufferType {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Video,
            2 => Self::AudioApp,
            3 => Self::AudioMic,
            other => Self::Unknown(other),
        }
    }

    /// Returns the raw `RPSampleBufferType` integer.
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Video => 1,
            Self::AudioApp => 2,
            Self::AudioMic => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

/// A sample buffer produced by `ReplayKit` capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSample {
    /// The underlying sample-buffer kind.
    pub sample_type: SampleBufferType,
    pub sample_buffer: CMSampleBuffer,
    /// Raw `CGImagePropertyOrientation` attachment when `ReplayKit` includes one.
    pub video_orientation: Option<u32>,
}

/// Events emitted by the sample-buffer capture bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureEvent {
    /// A captured sample buffer.
    Sample(CaptureSample),
    /// A capture callback error.
    Error(ReplayKitError),
}

/// Trait implemented by Rust capture delegates.
pub trait SampleBufferDelegate: Send + Sync + 'static {
    /// Handles the next capture event.
    fn handle_event(&self, event: CaptureEvent);
}

impl<F> SampleBufferDelegate for F
where
    F: Fn(CaptureEvent) + Send + Sync + 'static,
{
    fn handle_event(&self, event: CaptureEvent) {
        self(event);
    }
}

/// RAII guard for an active `ReplayKit` sample-buffer capture session.
pub struct SampleBufferCaptureSession {
    recorder: ScreenRecorder,
    context: CallbackContext<CaptureHandler>,
    stopped: bool,
}

impl SampleBufferCaptureSession {
    /// Whether the sample-buffer capture bridge is available on this platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_sample_buffer_delegate_is_supported() }
    }

    /// Stops the capture session and releases the delegate callback.
    pub fn stop(mut self) -> Result<(), ReplayKitError> {
        self.stopped = true;
        self.stop_capture()
    }

    fn stop_capture(&self) -> Result<(), ReplayKitError> {
        self.context.deactivate();
        let mut err: *mut c_char = ptr::null_mut();
        let rc =
            unsafe { ffi::rk_screen_recorder_stop_capture(self.recorder.as_ptr(), &raw mut err) };
        result_from_status(rc, err)
    }
}

impl Drop for SampleBufferCaptureSession {
    fn drop(&mut self) {
        if !self.stopped {
            let _ = self.stop_capture();
        }
    }
}

impl std::fmt::Debug for SampleBufferCaptureSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleBufferCaptureSession")
            .field("stopped", &self.stopped)
            .finish_non_exhaustive()
    }
}

unsafe extern "C" fn sample_capture_trampoline(
    context: *mut c_void,
    event_kind: i32,
    buffer_type: i32,
    sample_buffer: *mut c_void,
    has_orientation: bool,
    orientation: u32,
    error_json: *mut c_char,
) {
    let sample_buffer = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let error_message = unsafe { take_string(error_json) };
    let event = match (event_kind, sample_buffer) {
        (SAMPLE_EVENT, Some(sample_buffer)) => CaptureEvent::Sample(CaptureSample {
            sample_type: SampleBufferType::from_raw(buffer_type),
            sample_buffer,
            video_orientation: has_orientation.then_some(orientation),
        }),
        (SAMPLE_EVENT, None) => CaptureEvent::Error(ReplayKitError::Unknown(
            "sample-buffer event arrived without a sample buffer".into(),
        )),
        (ERROR_EVENT, _) => CaptureEvent::Error(crate::error::from_message(
            &error_message.unwrap_or_else(|| "sample-buffer capture failed".into()),
        )),
        (other, _) => CaptureEvent::Error(ReplayKitError::Unknown(format!(
            "unknown sample-buffer event kind: {other}"
        ))),
    };
    unsafe {
        CallbackContext::<CaptureHandler>::with(
            context,
            "replaykit::sample_buffer_delegate::sample_capture_trampoline",
            |delegate| delegate.handle_event(event),
        )
    };
}

impl ScreenRecorder {
    /// Starts `ReplayKit` sample-buffer capture and forwards events to the supplied delegate.
    pub fn start_capture<D>(
        &self,
        delegate: D,
    ) -> Result<SampleBufferCaptureSession, ReplayKitError>
    where
        D: SampleBufferDelegate,
    {
        let delegate: CaptureHandler = Box::new(delegate);
        let context = CallbackContext::new(delegate);
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_start_capture(
                self.as_ptr(),
                sample_capture_trampoline,
                context.as_ptr(),
                CallbackContext::<CaptureHandler>::RETAIN,
                CallbackContext::<CaptureHandler>::RELEASE,
                &raw mut err,
            )
        };
        if rc == crate::ffi::status::OK {
            Ok(SampleBufferCaptureSession {
                recorder: self.retained(),
                context,
                stopped: false,
            })
        } else {
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }
}

#[cfg(test)]
mod tests {
    use core::ffi::{c_char, c_void};
    use std::ptr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use apple_cf::cm::CMSampleBuffer;
    use doom_fish_utils::callback_context::CallbackContext;

    use super::{
        sample_capture_trampoline, CaptureEvent, CaptureHandler, SampleBufferType, ERROR_EVENT,
        SAMPLE_EVENT,
    };
    use crate::error::{RecordingErrorCode, ReplayKitError};

    extern "C" {
        fn CMSampleBufferCreate(
            allocator: *const c_void,
            data_buffer: *mut c_void,
            data_ready: u8,
            make_data_ready_callback: *const c_void,
            make_data_ready_refcon: *mut c_void,
            format_description: *mut c_void,
            num_samples: isize,
            num_sample_timing_entries: isize,
            sample_timing_array: *const c_void,
            num_sample_size_entries: isize,
            sample_size_array: *const usize,
            sample_buffer_out: *mut *mut c_void,
        ) -> i32;
        fn CFRetain(cf: *const c_void) -> *const c_void;
        fn CFGetRetainCount(cf: *const c_void) -> isize;
        fn strdup(value: *const c_char) -> *mut c_char;
    }

    fn empty_sample_buffer() -> CMSampleBuffer {
        let mut raw = ptr::null_mut();
        let status = unsafe {
            CMSampleBufferCreate(
                ptr::null(),
                ptr::null_mut(),
                1,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                0,
                ptr::null(),
                0,
                ptr::null(),
                &raw mut raw,
            )
        };
        assert_eq!(status, 0);
        unsafe { CMSampleBuffer::from_raw(raw) }.expect("CMSampleBufferCreate returned NULL")
    }

    fn plus_one(buffer: &CMSampleBuffer) -> *mut c_void {
        unsafe { CFRetain(buffer.as_ptr()) }.cast_mut()
    }

    fn retain_count(buffer: &CMSampleBuffer) -> isize {
        unsafe { CFGetRetainCount(buffer.as_ptr()) }
    }

    fn take_events(events: &Mutex<Vec<CaptureEvent>>) -> Vec<CaptureEvent> {
        std::mem::take(&mut *events.lock().unwrap())
    }

    fn recording_context() -> (CallbackContext<CaptureHandler>, Arc<Mutex<Vec<CaptureEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let handler: CaptureHandler =
            Box::new(move |event: CaptureEvent| sink.lock().unwrap().push(event));
        (CallbackContext::new(handler), events)
    }

    #[test]
    fn sample_events_hand_the_retained_buffer_to_the_delegate() {
        let buffer = empty_sample_buffer();
        let (context, events) = recording_context();

        unsafe {
            sample_capture_trampoline(
                context.as_ptr(),
                SAMPLE_EVENT,
                1,
                plus_one(&buffer),
                true,
                6,
                ptr::null_mut(),
            );
        }

        let received = take_events(&events);
        assert_eq!(received.len(), 1);
        let CaptureEvent::Sample(sample) = &received[0] else {
            panic!("expected a sample event, got {:?}", received[0]);
        };
        assert_eq!(sample.sample_type, SampleBufferType::Video);
        assert_eq!(sample.sample_buffer, buffer);
        assert_eq!(sample.video_orientation, Some(6));
        assert_eq!(sample.sample_buffer.num_samples(), 0);
        assert_eq!(retain_count(&buffer), 2);

        drop(received);
        assert_eq!(retain_count(&buffer), 1);
    }

    #[test]
    fn audio_samples_without_orientation_keep_their_type() {
        let buffer = empty_sample_buffer();
        let (context, events) = recording_context();

        unsafe {
            sample_capture_trampoline(
                context.as_ptr(),
                SAMPLE_EVENT,
                3,
                plus_one(&buffer),
                false,
                0,
                ptr::null_mut(),
            );
        }

        let received = take_events(&events);
        let CaptureEvent::Sample(sample) = &received[0] else {
            panic!("expected a sample event, got {:?}", received[0]);
        };
        assert_eq!(sample.sample_type, SampleBufferType::AudioMic);
        assert_eq!(sample.video_orientation, None);
    }

    #[test]
    fn deactivated_context_releases_the_buffer_without_calling_the_delegate() {
        let buffer = empty_sample_buffer();
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let handler: CaptureHandler = Box::new(move |_: CaptureEvent| {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        let context = CallbackContext::new(handler);
        context.deactivate();

        unsafe {
            sample_capture_trampoline(
                context.as_ptr(),
                SAMPLE_EVENT,
                2,
                plus_one(&buffer),
                false,
                0,
                ptr::null_mut(),
            );
        }

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(retain_count(&buffer), 1);
    }

    #[test]
    fn error_events_carry_framework_errors() {
        let (context, events) = recording_context();
        let payload = c"{\"kind\":\"framework\",\"domain\":\"RPRecordingErrorDomain\",\"code\":-5801,\"localizedDescription\":\"declined\"}";

        unsafe {
            sample_capture_trampoline(
                context.as_ptr(),
                ERROR_EVENT,
                0,
                ptr::null_mut(),
                false,
                0,
                strdup(payload.as_ptr()),
            );
        }

        let received = take_events(&events);
        let CaptureEvent::Error(ReplayKitError::Framework(error)) = &received[0] else {
            panic!("expected a framework error, got {:?}", received[0]);
        };
        assert_eq!(error.recording_code(), Some(RecordingErrorCode::UserDeclined));
        assert_eq!(error.localized_description, "declined");
    }

    #[test]
    fn sample_events_without_a_buffer_become_errors() {
        let (context, events) = recording_context();

        unsafe {
            sample_capture_trampoline(
                context.as_ptr(),
                SAMPLE_EVENT,
                1,
                ptr::null_mut(),
                false,
                0,
                ptr::null_mut(),
            );
        }

        let received = take_events(&events);
        assert!(matches!(&received[0], CaptureEvent::Error(ReplayKitError::Unknown(_))));
    }
}
