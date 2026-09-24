use std::ffi::{c_char, c_void, CStr};
use std::ptr;

use replaykit::{
    BroadcastExtensionContext, BroadcastHandler, BroadcastSampleHandler, ReplayKitError,
    ReplayKitFrameworkError, RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY,
};
use serde_json::json;

extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_msgSend();
}

struct TestObject(*mut c_void);

impl TestObject {
    fn new(class: &CStr) -> Self {
        let class = unsafe { objc_getClass(class.as_ptr()) };
        assert!(!class.is_null());
        let object = send(class, c"new");
        assert!(!object.is_null());
        Self(object)
    }

    fn retain_count(&self) -> usize {
        let send = unsafe {
            std::mem::transmute::<
                unsafe extern "C" fn(),
                unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize,
            >(objc_msgSend)
        };
        unsafe { send(self.0, sel_registerName(c"retainCount".as_ptr())) }
    }
}

impl Drop for TestObject {
    fn drop(&mut self) {
        send(self.0, c"release");
    }
}

fn send(receiver: *mut c_void, selector: &CStr) -> *mut c_void {
    let send = unsafe {
        std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
        >(objc_msgSend)
    };
    unsafe { send(receiver, sel_registerName(selector.as_ptr())) }
}

const fn is_invalid_argument<T>(result: &Result<T, ReplayKitError>) -> bool {
    matches!(result, Err(ReplayKitError::InvalidArgument(_)))
}

#[test]
fn broadcast_extension_support_is_reported() {
    assert!(BroadcastExtensionContext::is_supported_on_current_platform());
    assert!(BroadcastHandler::is_supported_on_current_platform());
    assert!(BroadcastSampleHandler::is_supported_on_current_platform());
}

#[test]
fn extension_objects_are_required() {
    assert!(is_invalid_argument(&unsafe {
        BroadcastExtensionContext::from_raw_borrowed(ptr::null_mut())
    }));
    assert!(is_invalid_argument(&unsafe {
        BroadcastHandler::from_raw_borrowed(ptr::null_mut())
    }));
    assert!(is_invalid_argument(&unsafe {
        BroadcastSampleHandler::from_raw_borrowed(ptr::null_mut())
    }));
}

#[test]
fn extension_objects_must_have_the_expected_class() {
    let object = TestObject::new(c"NSObject");
    let context = TestObject::new(c"NSExtensionContext");
    let handler = TestObject::new(c"RPBroadcastHandler");
    let sample_handler = TestObject::new(c"RPBroadcastSampleHandler");

    assert!(is_invalid_argument(&unsafe { BroadcastExtensionContext::from_raw_borrowed(object.0) }));
    assert!(is_invalid_argument(&unsafe { BroadcastExtensionContext::from_raw_borrowed(handler.0) }));
    assert!(is_invalid_argument(&unsafe { BroadcastHandler::from_raw_borrowed(object.0) }));
    assert!(is_invalid_argument(&unsafe { BroadcastHandler::from_raw_borrowed(context.0) }));
    assert!(is_invalid_argument(&unsafe { BroadcastSampleHandler::from_raw_borrowed(object.0) }));
    assert!(is_invalid_argument(&unsafe { BroadcastSampleHandler::from_raw_borrowed(handler.0) }));

    let wrapped_context = unsafe { BroadcastExtensionContext::from_raw_borrowed(context.0) }
        .expect("NSExtensionContext is accepted");
    let wrapped_handler =
        unsafe { BroadcastHandler::from_raw_borrowed(handler.0) }.expect("RPBroadcastHandler");
    let sample_as_handler = unsafe { BroadcastHandler::from_raw_borrowed(sample_handler.0) }
        .expect("RPBroadcastSampleHandler is an RPBroadcastHandler");
    let wrapped_sample_handler =
        unsafe { BroadcastSampleHandler::from_raw_borrowed(sample_handler.0) }
            .expect("RPBroadcastSampleHandler is accepted");

    assert_eq!(wrapped_context.class_name(), "NSExtensionContext");
    assert_eq!(wrapped_handler.class_name(), "RPBroadcastHandler");
    assert_eq!(sample_as_handler.class_name(), "RPBroadcastSampleHandler");
    assert_eq!(
        wrapped_sample_handler.as_handler().class_name(),
        "RPBroadcastSampleHandler"
    );
}

#[test]
fn wrappers_retain_the_extension_object_until_dropped() {
    let context = TestObject::new(c"NSExtensionContext");
    let sample_handler = TestObject::new(c"RPBroadcastSampleHandler");
    let context_count = context.retain_count();
    let handler_count = sample_handler.retain_count();

    let wrapped_context = unsafe { BroadcastExtensionContext::from_raw_borrowed(context.0) }
        .expect("NSExtensionContext is accepted");
    let wrapped_handler = unsafe { BroadcastSampleHandler::from_raw_borrowed(sample_handler.0) }
        .expect("RPBroadcastSampleHandler is accepted");
    assert_eq!(context.retain_count(), context_count + 1);
    assert_eq!(sample_handler.retain_count(), handler_count + 1);

    drop(wrapped_context);
    drop(wrapped_handler);
    assert_eq!(context.retain_count(), context_count);
    assert_eq!(sample_handler.retain_count(), handler_count);
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
    let context = TestObject::new(c"NSExtensionContext");
    let handler = TestObject::new(c"RPBroadcastHandler");
    let context = unsafe { BroadcastExtensionContext::from_raw_borrowed(context.0) }
        .expect("NSExtensionContext is accepted");
    let handler =
        unsafe { BroadcastHandler::from_raw_borrowed(handler.0) }.expect("RPBroadcastHandler");

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
    let context = TestObject::new(c"NSExtensionContext");
    let sample_handler = TestObject::new(c"RPBroadcastSampleHandler");
    let context = unsafe { BroadcastExtensionContext::from_raw_borrowed(context.0) }
        .expect("NSExtensionContext is accepted");
    let sample_handler = unsafe { BroadcastSampleHandler::from_raw_borrowed(sample_handler.0) }
        .expect("RPBroadcastSampleHandler is accepted");
    let handler = sample_handler.as_handler();

    assert!(is_invalid_argument(&handler.update_broadcast_url("bad\0url")));
    assert!(is_invalid_argument(
        &handler.update_service_info(&json!(["not", "an", "object"]))
    ));
    assert!(is_invalid_argument(
        &context.complete_request_with_broadcast_url("")
    ));
    assert!(is_invalid_argument(&sample_handler.finish_broadcast_with_error(
        &ReplayKitFrameworkError {
            domain: "bad\0domain".into(),
            code: -5804,
            localized_description: "broadcast failed".into(),
        }
    )));
}
