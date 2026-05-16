#![allow(missing_docs, dead_code)]

use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_string_free(s: *mut c_char);

    // ── ScreenRecorder ────────────────────────────────────────────────────────
    pub fn rk_screen_recorder_shared() -> *mut c_void;
    pub fn rk_screen_recorder_release(ptr: *mut c_void);

    pub fn rk_screen_recorder_is_available(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_is_recording(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_is_microphone_enabled(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_is_camera_enabled(ptr: *mut c_void) -> bool;

    pub fn rk_screen_recorder_state_json(ptr: *mut c_void) -> *mut c_char;

    pub fn rk_screen_recorder_start_recording(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_stop_recording(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn rk_screen_recorder_set_delegate(
        recorder_ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, *const c_char),
        refcon: *mut c_void,
    ) -> *mut c_void;
    pub fn rk_screen_recorder_clear_delegate(recorder_ptr: *mut c_void, holder_ptr: *mut c_void);

    // ── BroadcastActivityController ───────────────────────────────────────────
    pub fn rk_broadcast_activity_controller_show(
        origin_x: f64,
        origin_y: f64,
        window_ptr: *mut c_void,
        preferred_extension_json: *const c_char,
        refcon: *mut c_void,
        completion_callback: unsafe extern "C" fn(
            refcon: *mut c_void,
            controller_ptr: *mut c_void,
            error_json: *mut c_char,
        ),
    );

    // ── BroadcastController ───────────────────────────────────────────────────
    pub fn rk_broadcast_controller_release(ptr: *mut c_void);
    pub fn rk_broadcast_controller_is_broadcasting(ptr: *mut c_void) -> bool;
    pub fn rk_broadcast_controller_is_paused(ptr: *mut c_void) -> bool;
    pub fn rk_broadcast_controller_broadcast_url(ptr: *mut c_void) -> *mut c_char;
    pub fn rk_broadcast_controller_start(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn rk_broadcast_controller_finish(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn rk_broadcast_controller_pause(ptr: *mut c_void);
    pub fn rk_broadcast_controller_resume(ptr: *mut c_void);
}

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const TIMED_OUT: i32 = -2;
    pub const NOT_SUPPORTED: i32 = -3;
    pub const FRAMEWORK_ERROR: i32 = -4;
    pub const UNKNOWN: i32 = -99;
}
