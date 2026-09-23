use replaykit::{
    BroadcastExtensionContext, BroadcastHandler, BroadcastSampleHandler, ReplayKitError,
    RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};
use serde_json::json;

#[test]
fn broadcast_extension_support_is_reported() {
    assert!(BroadcastExtensionContext::is_supported_on_current_platform());
    assert!(BroadcastHandler::is_supported_on_current_platform());
}

#[test]
fn broadcast_sample_handler_reports_unsupported() {
    assert!(!BroadcastSampleHandler::is_supported_on_current_platform());
    let reason = BroadcastSampleHandler::unsupported_reason();
    assert!(reason.contains("processSampleBuffer"));
    let Err(error) = BroadcastSampleHandler::new();
    assert_eq!(error, ReplayKitError::NotSupported(reason));
}

#[test]
fn broadcast_extension_symbols_are_constructible() {
    let context = BroadcastExtensionContext::new();
    let handler = BroadcastHandler::new();

    assert_eq!(context.class_name(), "NSExtensionContext");
    assert_eq!(handler.class_name(), "RPBroadcastHandler");
}

#[test]
fn bundle_identifier_key_matches_framework_value() {
    assert_eq!(
        RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
        "RPApplicationInfoBundleIdentifier"
    );
}

#[test]
fn broadcast_extension_methods_accept_json_payloads() {
    let context = BroadcastExtensionContext::new();
    let handler = BroadcastHandler::new();

    context
        .complete_request_with_broadcast_url_and_setup_info(
            "https://example.com/broadcast",
            &json!({"quality": "high"}),
        )
        .expect("complete request should accept JSON setup info");
    handler
        .update_service_info(&json!({"status": "ready", "viewers": 3}))
        .expect("service info should accept JSON objects");
    handler
        .update_broadcast_url("https://example.com/live")
        .expect("broadcast URL should parse");
}

#[test]
fn broadcast_extension_methods_reject_invalid_input() {
    let context = BroadcastExtensionContext::new();
    let handler = BroadcastHandler::new();

    assert!(matches!(
        handler.update_broadcast_url("bad\0url"),
        Err(ReplayKitError::InvalidArgument(_))
    ));
    assert!(matches!(
        handler.update_service_info(&json!(["not", "an", "object"])),
        Err(ReplayKitError::InvalidArgument(_))
    ));
    assert!(matches!(
        context.complete_request_with_broadcast_url(""),
        Err(ReplayKitError::InvalidArgument(_))
    ));
}
