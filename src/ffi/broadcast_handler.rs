use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_handler_is_supported() -> bool;
    pub fn rk_broadcast_handler_new() -> *mut c_void;
    pub fn rk_broadcast_handler_update_service_info(
        ptr: *mut c_void,
        service_info_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_handler_update_broadcast_url(
        ptr: *mut c_void,
        broadcast_url: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
