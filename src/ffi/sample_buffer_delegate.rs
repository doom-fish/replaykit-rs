use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_sample_buffer_delegate_is_supported() -> bool;
    pub fn rk_screen_recorder_start_capture(
        ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, i32, *mut c_char),
        refcon: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_stop_capture(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
}
