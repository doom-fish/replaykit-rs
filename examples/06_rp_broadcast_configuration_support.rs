use replaykit::BroadcastConfiguration;

fn main() {
    println!(
        "broadcast_configuration_supported={}",
        BroadcastConfiguration::is_supported_on_current_platform()
    );
    println!("reason={}", BroadcastConfiguration::unsupported_reason());
}
