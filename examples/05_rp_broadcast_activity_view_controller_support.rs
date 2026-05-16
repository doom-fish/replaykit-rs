use replaykit::{BroadcastActivityControllerHandle, BroadcastActivityViewController};

fn main() {
    println!(
        "macos_broadcast_activity_controller_supported={}",
        BroadcastActivityControllerHandle::is_supported_on_current_platform()
    );
    println!(
        "ios_broadcast_activity_view_controller_supported={}",
        BroadcastActivityViewController::is_supported_on_current_platform()
    );
    println!(
        "reason={}",
        BroadcastActivityViewController::unsupported_reason()
    );
}
