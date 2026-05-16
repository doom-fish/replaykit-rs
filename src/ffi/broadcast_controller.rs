use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_broadcast_controller_is_supported() -> bool;
    pub fn rk_broadcast_controller_release(ptr: *mut c_void);
    pub fn rk_broadcast_controller_is_broadcasting(ptr: *mut c_void) -> bool;
    pub fn rk_broadcast_controller_is_paused(ptr: *mut c_void) -> bool;
    pub fn rk_broadcast_controller_broadcast_url(ptr: *mut c_void) -> *mut c_char;
    pub fn rk_broadcast_controller_service_info_json(ptr: *mut c_void) -> *mut c_char;
    pub fn rk_broadcast_controller_start(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn rk_broadcast_controller_finish(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn rk_broadcast_controller_pause(ptr: *mut c_void);
    pub fn rk_broadcast_controller_resume(ptr: *mut c_void);
    pub fn rk_broadcast_controller_set_delegate(
        controller_ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, i32, *mut c_char),
        refcon: *mut c_void,
    ) -> *mut c_void;
    pub fn rk_broadcast_controller_clear_delegate(
        controller_ptr: *mut c_void,
        holder_ptr: *mut c_void,
    );
}
