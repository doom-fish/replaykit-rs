use core::ffi::{c_char, c_void};

extern "C" {
    pub fn rk_screen_recorder_shared() -> *mut c_void;
    pub fn rk_screen_recorder_release(ptr: *mut c_void);

    pub fn rk_screen_recorder_is_available(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_is_recording(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_is_microphone_enabled(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_set_microphone_enabled(ptr: *mut c_void, enabled: bool);
    pub fn rk_screen_recorder_is_camera_enabled(ptr: *mut c_void) -> bool;
    pub fn rk_screen_recorder_set_camera_enabled(ptr: *mut c_void, enabled: bool);
    pub fn rk_screen_recorder_camera_position(ptr: *mut c_void) -> i32;
    pub fn rk_screen_recorder_set_camera_position(ptr: *mut c_void, camera_position: i32);
    pub fn rk_screen_recorder_camera_preview_view(ptr: *mut c_void) -> *mut c_void;

    pub fn rk_screen_recorder_state_json(ptr: *mut c_void) -> *mut c_char;

    pub fn rk_screen_recorder_start_recording(ptr: *mut c_void, out_error: *mut *mut c_char)
        -> i32;
    pub fn rk_screen_recorder_stop_recording(ptr: *mut c_void, out_error: *mut *mut c_char) -> i32;
    pub fn rk_screen_recorder_stop_recording_with_preview(
        ptr: *mut c_void,
        out_preview_controller: *mut *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_stop_recording_with_output_url(
        ptr: *mut c_void,
        output_path: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_discard_recording(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_start_clip_buffering(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_stop_clip_buffering(
        ptr: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn rk_screen_recorder_export_clip_to_output_url(
        ptr: *mut c_void,
        output_path: *const c_char,
        duration_seconds: f64,
        out_error: *mut *mut c_char,
    ) -> i32;

    pub fn rk_screen_recorder_set_delegate(
        recorder_ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, *const c_char),
        refcon: *mut c_void,
    ) -> *mut c_void;
    pub fn rk_screen_recorder_clear_delegate(recorder_ptr: *mut c_void, holder_ptr: *mut c_void);

    pub fn rk_screen_recorder_set_detailed_delegate(
        recorder_ptr: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, i32, bool, *mut c_void, *mut c_char),
        refcon: *mut c_void,
    ) -> *mut c_void;
    pub fn rk_screen_recorder_clear_detailed_delegate(
        recorder_ptr: *mut c_void,
        holder_ptr: *mut c_void,
    );

    pub fn rk_ns_view_is_hidden(ptr: *mut c_void) -> bool;
}
