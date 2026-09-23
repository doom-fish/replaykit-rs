use std::ffi::c_void;
use std::ptr;

use apple_cf::cm::CMSampleBuffer;
use replaykit::{CaptureEvent, CaptureSample, SampleBufferCaptureSession, SampleBufferType};

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

#[test]
fn sample_buffer_type_raw_values_are_stable() {
    assert_eq!(SampleBufferType::Video.as_raw(), 1);
    assert_eq!(SampleBufferType::AudioApp.as_raw(), 2);
    assert_eq!(SampleBufferType::AudioMic.as_raw(), 3);
}

#[test]
fn capture_session_support_is_reported() {
    assert!(SampleBufferCaptureSession::is_supported_on_current_platform());
}

#[test]
fn capture_events_hold_the_sample_buffer() {
    let buffer = empty_sample_buffer();
    let sample = CaptureSample {
        sample_type: SampleBufferType::Video,
        sample_buffer: buffer.clone(),
        video_orientation: Some(1),
    };
    let event = CaptureEvent::Sample(sample.clone());
    match event {
        CaptureEvent::Sample(inner) => {
            assert_eq!(inner, sample);
            assert_eq!(inner.sample_buffer, buffer);
            assert!(inner.sample_buffer.data_is_ready());
            assert_eq!(inner.sample_buffer.num_samples(), 0);
        }
        CaptureEvent::Error(_) => panic!("expected sample event"),
    }
}
