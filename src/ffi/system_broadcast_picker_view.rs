use core::ffi::c_char;

extern "C" {
    pub fn rk_system_broadcast_picker_view_is_supported() -> bool;
    pub fn rk_system_broadcast_picker_view_unavailable_reason() -> *mut c_char;
}
