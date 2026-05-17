use core::ffi::{c_char, c_void};

extern "C" {
    /// Async start recording - calls callback with (`result_ptr`, `error_cstr`, `ctx`)
    pub fn rk_screen_recorder_start_recording_async(
        ptr: *mut c_void,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    /// Async stop recording - returns optional preview controller via callback
    pub fn rk_screen_recorder_stop_recording_async(
        ptr: *mut c_void,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    /// Async stop recording with output file
    pub fn rk_screen_recorder_stop_recording_with_output_async(
        ptr: *mut c_void,
        output_path: *const c_char,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    /// Async start capture - with sample callback and completion callback
    pub fn rk_screen_recorder_start_capture_async(
        ptr: *mut c_void,
        sample_cb: unsafe extern "C" fn(*mut c_void, i32, *const c_char),
        sample_ctx: *mut c_void,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    /// Async stop capture
    pub fn rk_screen_recorder_stop_capture_async(
        ptr: *mut c_void,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );

    /// Async discard recording
    pub fn rk_screen_recorder_discard_recording_async(
        ptr: *mut c_void,
        cb: unsafe extern "C" fn(*const c_void, *const i8, *mut c_void),
        ctx: *mut c_void,
    );
}
