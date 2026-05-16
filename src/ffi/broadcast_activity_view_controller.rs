use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_activity_controller_show(
        origin_x: f64,
        origin_y: f64,
        window_ptr: *mut c_void,
        preferred_extension: *const c_char,
        refcon: *mut c_void,
        completion_callback: unsafe extern "C" fn(
            refcon: *mut c_void,
            controller_ptr: *mut c_void,
            error_json: *mut c_char,
        ),
    );
    pub fn rk_broadcast_activity_view_controller_is_supported() -> bool;
    pub fn rk_broadcast_activity_view_controller_unavailable_reason() -> *mut c_char;
}
