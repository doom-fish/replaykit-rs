use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_string_free(s: *mut c_char);
    pub fn rk_object_release(ptr: *mut c_void);
    pub fn rk_object_class_name(ptr: *mut c_void) -> *mut c_char;
}
