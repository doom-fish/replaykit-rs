use replaykit::{SampleBufferCaptureSession, SampleBufferType};

fn main() {
    println!(
        "sample_buffer_capture_supported={}",
        SampleBufferCaptureSession::is_supported_on_current_platform()
    );
    println!("video_raw={}", SampleBufferType::Video.as_raw());
}
