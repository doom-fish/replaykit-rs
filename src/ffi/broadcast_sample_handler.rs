use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_sample_handler_is_supported() -> bool;
    pub fn rk_broadcast_sample_handler_retain(
        ptr: *mut c_void,
        out_handler: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_sample_handler_finish_broadcast_with_error(
        ptr: *mut c_void,
        domain: *const c_char,
        code: i64,
        localized_description: *const c_char,
    );
}
