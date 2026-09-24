use replaykit::{
    BroadcastExtensionContext, BroadcastHandler, BroadcastSampleHandler,
    RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};

fn main() {
    println!(
        "broadcast_extension_supported={}",
        BroadcastExtensionContext::is_supported_on_current_platform()
    );
    println!(
        "broadcast_handler_supported={}",
        BroadcastHandler::is_supported_on_current_platform()
    );
    println!(
        "sample_handler_supported={}",
        BroadcastSampleHandler::is_supported_on_current_platform()
    );
    println!(
        "application_info_bundle_identifier_key={RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY}"
    );
}
