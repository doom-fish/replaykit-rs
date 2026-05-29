use core::ffi::{c_char, c_void};
use std::ptr;

use serde::Deserialize;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{
    context_release_cb, context_retain_cb, parse_json_ptr, result_from_status, take_string,
    CallbackBox,
};
use crate::screen_recorder::ScreenRecorder;

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

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct CaptureSamplePayload {
    #[serde(rename = "bufferType")]
    buffer_type: i32,
    #[serde(rename = "numSamples")]
    num_samples: usize,
    #[serde(rename = "dataIsReady")]
    data_is_ready: bool,
    #[serde(rename = "presentationTimeSeconds")]
    presentation_time_seconds: Option<f64>,
    #[serde(rename = "durationSeconds")]
    duration_seconds: Option<f64>,
    #[serde(rename = "videoOrientation")]
    video_orientation: Option<u32>,
}

/// Metadata extracted from a `CMSampleBufferRef` produced by `ReplayKit` capture.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureSample {
    /// The underlying sample-buffer kind.
    pub sample_type: SampleBufferType,
    /// The number of samples in the buffer.
    pub num_samples: usize,
    /// Whether the buffer data is ready for consumption.
    pub data_is_ready: bool,
    /// Presentation timestamp in seconds when available.
    pub presentation_time_seconds: Option<f64>,
    /// Duration in seconds when available.
    pub duration_seconds: Option<f64>,
    /// Raw `CGImagePropertyOrientation` attachment when `ReplayKit` includes one.
    pub video_orientation: Option<u32>,
}

impl From<CaptureSamplePayload> for CaptureSample {
    fn from(value: CaptureSamplePayload) -> Self {
        Self {
            sample_type: SampleBufferType::from_raw(value.buffer_type),
            num_samples: value.num_samples,
            data_is_ready: value.data_is_ready,
            presentation_time_seconds: value.presentation_time_seconds,
            duration_seconds: value.duration_seconds,
            video_orientation: value.video_orientation,
        }
    }
}

/// Events emitted by the sample-buffer capture bridge.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureEvent {
    /// A captured sample buffer.
    Sample(CaptureSample),
    /// A capture callback error.
    Error(ReplayKitError),
}

/// Trait implemented by Rust capture delegates.
pub trait SampleBufferDelegate: Send + 'static {
    /// Handles the next capture event.
    fn handle_event(&self, event: CaptureEvent);
}

impl<F> SampleBufferDelegate for F
where
    F: Fn(CaptureEvent) + Send + 'static,
{
    fn handle_event(&self, event: CaptureEvent) {
        self(event);
    }
}

/// RAII guard for an active `ReplayKit` sample-buffer capture session.
pub struct SampleBufferCaptureSession {
    recorder_ptr: *mut c_void,
    context: *mut CallbackBox<dyn SampleBufferDelegate>,
    stopped: bool,
}

unsafe impl Send for SampleBufferCaptureSession {}
unsafe impl Sync for SampleBufferCaptureSession {}

impl SampleBufferCaptureSession {
    /// Whether the sample-buffer capture bridge is available on this platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_sample_buffer_delegate_is_supported() }
    }

    /// Stops the capture session and releases the delegate callback.
    pub fn stop(mut self) -> Result<(), ReplayKitError> {
        let result = self.stop_inner();
        self.free_delegate();
        self.stopped = true;
        result
    }

    fn stop_inner(&mut self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_screen_recorder_stop_capture(self.recorder_ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    fn free_delegate(&mut self) {
        if !self.context.is_null() {
            // Release this session's reference. The Swift capture closure holds
            // its own +1 reference until ReplayKit invokes the completion
            // handler, so the delegate is freed only once no capture callback
            // can still run on the capture queue.
            unsafe { CallbackBox::release(self.context) };
            self.context = ptr::null_mut();
        }
    }
}

impl Drop for SampleBufferCaptureSession {
    fn drop(&mut self) {
        if self.stopped {
            return;
        }
        let _ = self.stop_inner();
        self.free_delegate();
    }
}

impl std::fmt::Debug for SampleBufferCaptureSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SampleBufferCaptureSession")
            .finish_non_exhaustive()
    }
}

unsafe extern "C" fn sample_capture_trampoline(
    refcon: *mut c_void,
    event_kind: i32,
    payload: *mut c_char,
) {
    let event = match event_kind {
        1 => unsafe {
            parse_json_ptr::<CaptureSamplePayload>(payload, "capture sample event")
                .map_or_else(CaptureEvent::Error, |sample| {
                    CaptureEvent::Sample(sample.into())
                })
        },
        2 => {
            let message = unsafe { take_string(payload) }
                .unwrap_or_else(|| "sample-buffer capture failed".into());
            CaptureEvent::Error(crate::error::from_message(&message))
        }
        _ => {
            let message = unsafe { take_string(payload) }
                .unwrap_or_else(|| format!("unknown sample-buffer event kind: {event_kind}"));
            CaptureEvent::Error(ReplayKitError::Unknown(message))
        }
    };
    let delegate = unsafe { CallbackBox::<dyn SampleBufferDelegate>::handler(refcon.cast()) };
    doom_fish_utils::panic_safe::catch_user_panic(
        "replaykit::sample_buffer_delegate::sample_capture_trampoline",
        || delegate.handle_event(event),
    );
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
        let boxed: Box<dyn SampleBufferDelegate> = Box::new(delegate);
        let context = CallbackBox::into_raw(boxed);
        let refcon = context.cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_screen_recorder_start_capture(
                self.as_ptr(),
                sample_capture_trampoline,
                refcon,
                context_retain_cb::<dyn SampleBufferDelegate>,
                context_release_cb::<dyn SampleBufferDelegate>,
                &raw mut err,
            )
        };
        if rc == crate::ffi::status::OK {
            Ok(SampleBufferCaptureSession {
                recorder_ptr: self.as_ptr(),
                context,
                stopped: false,
            })
        } else {
            unsafe { CallbackBox::release(context) };
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }
}
