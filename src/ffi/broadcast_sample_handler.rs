use core::ffi::c_char;

extern "C" {
    pub fn rk_broadcast_sample_handler_is_supported() -> bool;
    pub fn rk_broadcast_sample_handler_unavailable_reason() -> *mut c_char;
}
