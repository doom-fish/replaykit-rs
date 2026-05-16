use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_extension_context_is_supported() -> bool;
    pub fn rk_broadcast_extension_context_new() -> *mut c_void;
    pub fn rk_broadcast_extension_context_load_application_info_json(
        ptr: *mut c_void,
        out_json: *mut *mut c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_extension_context_complete_request_with_broadcast_url(
        ptr: *mut c_void,
        broadcast_url: *const c_char,
        setup_info_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
