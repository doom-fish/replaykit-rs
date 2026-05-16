use core::ffi::c_char;

extern "C" {
    pub fn rk_broadcast_configuration_is_supported() -> bool;
    pub fn rk_broadcast_configuration_unavailable_reason() -> *mut c_char;
}
