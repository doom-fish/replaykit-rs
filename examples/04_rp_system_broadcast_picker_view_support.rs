use replaykit::SystemBroadcastPickerView;

fn main() {
    println!(
        "system_broadcast_picker_view_supported={}",
        SystemBroadcastPickerView::is_supported_on_current_platform()
    );
    println!("reason={}", SystemBroadcastPickerView::unsupported_reason());
}
