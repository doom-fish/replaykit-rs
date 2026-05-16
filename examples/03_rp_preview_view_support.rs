use replaykit::PreviewViewController;

fn main() {
    println!(
        "preview_view_controller_supported={}",
        PreviewViewController::is_supported_on_current_platform()
    );
}
