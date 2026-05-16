use core::ffi::{c_char, c_void};
use std::ptr;

use crate::broadcast_controller::BroadcastController;
use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{cstring_from_str, take_string};

/// Result returned through the callback passed to
/// [`BroadcastActivityControllerHandle::show`].
pub type ShowResult = Result<BroadcastController, ReplayKitError>;

/// Wraps the macOS `RPBroadcastActivityController` picker flow.
///
/// The iOS `RPBroadcastActivityViewController` type itself is unavailable on macOS;
/// this handle exposes the native macOS equivalent.
pub struct BroadcastActivityControllerHandle;

impl BroadcastActivityControllerHandle {
    /// Whether the macOS broadcast-activity controller flow is available.
    pub fn is_supported_on_current_platform() -> bool {
        BroadcastController::is_supported_on_current_platform()
    }

    /// Presents the system broadcast-picker sheet at `origin` in the application
    /// window and calls `completion` when done.
    pub fn show(
        origin: (f64, f64),
        preferred_extension: Option<&str>,
        completion: impl FnOnce(ShowResult) + Send + 'static,
    ) {
        let boxed: Box<dyn FnOnce(ShowResult) + Send + 'static> = Box::new(completion);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();

        let ext_cstr = match preferred_extension {
            Some(value) => match cstring_from_str(value, "preferred broadcast extension") {
                Ok(value) => Some(value),
                Err(error) => {
                    let boxed = unsafe {
                        Box::from_raw(refcon.cast::<Box<dyn FnOnce(ShowResult) + Send + 'static>>())
                    };
                    (*boxed)(Err(error));
                    return;
                }
            },
            None => None,
        };
        let ext_ptr: *const c_char = ext_cstr
            .as_ref()
            .map_or(ptr::null(), |value| value.as_ptr());

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
    let boxed = Box::from_raw(refcon.cast::<Box<dyn FnOnce(ShowResult) + Send + 'static>>());
    let result = if error_json.is_null() && !controller_ptr.is_null() {
        Ok(BroadcastController::from_ptr(controller_ptr))
    } else {
        let message = take_string(error_json).unwrap_or_else(|| "unknown error".into());
        Err(crate::error::from_message(&message))
    };
    (*boxed)(result);
}

/// Explicit macOS wrapper for the iOS-only `RPBroadcastActivityViewController` area.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BroadcastActivityViewController;

impl BroadcastActivityViewController {
    /// Whether the iOS broadcast-activity view controller is available.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_activity_view_controller_is_supported() }
    }

    /// Returns the macOS-specific unavailability reason.
    pub fn unsupported_reason() -> String {
        let ptr = unsafe { ffi::rk_broadcast_activity_view_controller_unavailable_reason() };
        unsafe { take_string(ptr) }
            .unwrap_or_else(|| "RPBroadcastActivityViewController is unavailable on macOS".into())
    }

    /// Attempts to load the iOS broadcast-activity view controller.
    pub fn load() -> Result<Self, ReplayKitError> {
        Err(ReplayKitError::NotSupported(Self::unsupported_reason()))
    }
}
