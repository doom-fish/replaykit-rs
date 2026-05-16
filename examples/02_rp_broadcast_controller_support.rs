use replaykit::{BroadcastActivityControllerHandle, BroadcastController};

fn main() {
    println!(
        "broadcast_controller_supported={}",
        BroadcastController::is_supported_on_current_platform()
    );
    println!(
        "broadcast_activity_controller_supported={}",
        BroadcastActivityControllerHandle::is_supported_on_current_platform()
    );
}
