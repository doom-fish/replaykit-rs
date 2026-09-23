use replaykit::{
    BroadcastExtensionContext, BroadcastHandler, BroadcastSampleHandler,
    RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};

fn main() {
    let context = BroadcastExtensionContext::new();
    let handler = BroadcastHandler::new();

    println!(
        "broadcast_extension_supported={}",
        BroadcastExtensionContext::is_supported_on_current_platform()
    );
    println!("context_class={}", context.class_name());
    println!("handler_class={}", handler.class_name());
    println!(
        "sample_handler_supported={}",
        BroadcastSampleHandler::is_supported_on_current_platform()
    );
    println!(
        "sample_handler_reason={}",
        BroadcastSampleHandler::unsupported_reason()
    );
    println!(
        "application_info_bundle_identifier_key={RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY}"
    );
}
