use replaykit::{
    BroadcastExtensionContext, BroadcastHandler, BroadcastSampleHandler,
    ReplayKitFrameworkError, RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};
use serde_json::json;

#[test]
fn broadcast_extension_support_is_reported() {
    assert!(BroadcastExtensionContext::is_supported_on_current_platform());
    assert!(BroadcastHandler::is_supported_on_current_platform());
    assert!(BroadcastSampleHandler::is_supported_on_current_platform());
}

#[test]
fn broadcast_extension_symbols_are_constructible() {
    let context = BroadcastExtensionContext::new();
    let handler = BroadcastHandler::new();
    let sample_handler = BroadcastSampleHandler::new();

    assert_eq!(context.class_name(), "NSExtensionContext");
    assert_eq!(handler.class_name(), "RPBroadcastHandler");
    assert_eq!(sample_handler.class_name(), "RPBroadcastSampleHandler");
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
    let sample_handler = BroadcastSampleHandler::new();

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
    sample_handler
        .broadcast_started_with_setup_info(&json!({"token": "abc"}))
        .expect("setup info should accept JSON objects");
    sample_handler.broadcast_paused();
    sample_handler.broadcast_resumed();
    sample_handler.broadcast_finished();
    sample_handler
        .broadcast_annotated_with_application_info(&json!({
            RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY: "com.example.broadcast"
        }))
        .expect("application info should accept JSON objects");
    sample_handler
        .finish_broadcast_with_error(&ReplayKitFrameworkError {
            domain: "RPRecordingErrorDomain".into(),
            code: -5804,
            localized_description: "broadcast failed".into(),
        })
        .expect("framework error payload should bridge to NSError");
}
