use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_sample_handler_is_supported() -> bool;
    pub fn rk_broadcast_sample_handler_new() -> *mut c_void;
    pub fn rk_broadcast_sample_handler_update_service_info(
        ptr: *mut c_void,
        service_info_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_sample_handler_update_broadcast_url(
        ptr: *mut c_void,
        broadcast_url: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_sample_handler_broadcast_started(
        ptr: *mut c_void,
        setup_info_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_sample_handler_broadcast_paused(ptr: *mut c_void);
    pub fn rk_broadcast_sample_handler_broadcast_resumed(ptr: *mut c_void);
    pub fn rk_broadcast_sample_handler_broadcast_finished(ptr: *mut c_void);
    pub fn rk_broadcast_sample_handler_broadcast_annotated_with_application_info(
        ptr: *mut c_void,
        application_info_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_broadcast_sample_handler_finish_broadcast_with_error(
        ptr: *mut c_void,
        domain: *const c_char,
        code: i64,
        localized_description: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
