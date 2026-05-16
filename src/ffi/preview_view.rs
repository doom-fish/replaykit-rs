use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_preview_view_controller_is_supported() -> bool;
    pub fn rk_preview_view_controller_is_view_loaded(ptr: *mut c_void) -> bool;
    pub fn rk_preview_view_controller_set_delegate(
        controller_ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, i32, *mut c_char),
        refcon: *mut c_void,
    ) -> *mut c_void;
    pub fn rk_preview_view_controller_clear_delegate(
        controller_ptr: *mut c_void,
        holder_ptr: *mut c_void,
    );
}
