use core::ffi::{c_char, c_void};
use std::ptr;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::take_string;

// ── BroadcastActivityControllerHandle ────────────────────────────────────────

/// Result returned through the callback passed to
/// [`BroadcastActivityControllerHandle::show`].
pub type ShowResult = Result<BroadcastController, ReplayKitError>;

/// Wraps `RPBroadcastActivityController` (macOS 11+).
///
/// On iOS the corresponding type is `RPBroadcastActivityViewController`.  That
/// view-controller API is iOS-only; calling [`BroadcastActivityControllerHandle::show`]
/// on macOS 11+ uses the native picker sheet.
pub struct BroadcastActivityControllerHandle;

impl BroadcastActivityControllerHandle {
    /// Presents the system broadcast-picker sheet at `origin` in the application
    /// window and calls `completion` when done.
    ///
    /// - `origin`: where (in window coordinates, measured from the bottom-left
    ///   of the window) to place the picker.
    /// - `preferred_extension`: optional bundle identifier of the preferred
    ///   broadcast-extension service.
    /// - `completion`: called exactly once with the result.
    pub fn show(
        origin: (f64, f64),
        preferred_extension: Option<&str>,
        completion: impl FnOnce(ShowResult) + Send + 'static,
    ) {
        // Box the completion so we can pass a raw pointer as refcon.
        let boxed: Box<dyn FnOnce(ShowResult) + Send + 'static> = Box::new(completion);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();

        let ext_cstr = preferred_extension.and_then(|s| std::ffi::CString::new(s).ok());
        let ext_ptr: *const c_char = ext_cstr
            .as_ref()
            .map_or(ptr::null(), |s| s.as_ptr());

        unsafe {
            ffi::rk_broadcast_activity_controller_show(
                origin.0,
                origin.1,
                ptr::null_mut(),
                ext_ptr,
                refcon,
                show_trampoline,
            );
        }
    }
}

unsafe extern "C" fn show_trampoline(
    refcon: *mut c_void,
    controller_ptr: *mut c_void,
    error_json: *mut c_char,
) {
    // Reclaim the Box.
    let boxed = Box::from_raw(refcon.cast::<Box<dyn FnOnce(ShowResult) + Send + 'static>>());
    let result = if error_json.is_null() && !controller_ptr.is_null() {
        Ok(BroadcastController { ptr: controller_ptr })
    } else {
        let msg = take_string(error_json).unwrap_or_else(|| "unknown error".into());
        let err = serde_json::from_str::<serde_json::Value>(&msg)
            .ok()
            .and_then(|v| {
                Some(ReplayKitError::Framework(
                    crate::error::ReplayKitFrameworkError {
                        domain: v["domain"].as_str()?.to_owned(),
                        code: v["code"].as_i64()?,
                        localized_description: v["localizedDescription"]
                            .as_str()?
                            .to_owned(),
                    },
                ))
            })
            .unwrap_or(ReplayKitError::Unknown(msg));
        Err(err)
    };
    (*boxed)(result);
}

// ── BroadcastController ───────────────────────────────────────────────────────

/// A safe wrapper around `RPBroadcastController` (macOS 11+).
///
/// Obtained from the completion callback of
/// [`BroadcastActivityControllerHandle::show`].
pub struct BroadcastController {
    ptr: *mut c_void,
}

// Safety: RPBroadcastController is safe to access from multiple threads for
// property reads; we ensure exclusive mutation through &mut methods.
unsafe impl Send for BroadcastController {}
unsafe impl Sync for BroadcastController {}

impl BroadcastController {
    /// Whether a broadcast is currently active.
    pub fn is_broadcasting(&self) -> bool {
        unsafe { ffi::rk_broadcast_controller_is_broadcasting(self.ptr) }
    }

    /// Whether the broadcast is paused.
    pub fn is_paused(&self) -> bool {
        unsafe { ffi::rk_broadcast_controller_is_paused(self.ptr) }
    }

    /// The URL where the broadcast can be watched.
    pub fn broadcast_url(&self) -> String {
        let ptr = unsafe { ffi::rk_broadcast_controller_broadcast_url(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_default()
    }

    /// Starts the broadcast.
    pub fn start(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_broadcast_controller_start(self.ptr, &raw mut err) };
        if rc == crate::ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }

    /// Finishes the broadcast.
    pub fn finish(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_broadcast_controller_finish(self.ptr, &raw mut err) };
        if rc == crate::ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }

    /// Pauses the broadcast.
    pub fn pause(&self) {
        unsafe { ffi::rk_broadcast_controller_pause(self.ptr) };
    }

    /// Resumes a paused broadcast.
    pub fn resume(&self) {
        unsafe { ffi::rk_broadcast_controller_resume(self.ptr) };
    }
}

impl Drop for BroadcastController {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_broadcast_controller_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for BroadcastController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastController")
            .field("is_broadcasting", &self.is_broadcasting())
            .field("is_paused", &self.is_paused())
            .finish()
    }
}
