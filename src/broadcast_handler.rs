use core::ffi::{c_char, c_void};
use std::ptr;

use serde::Serialize;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, result_from_status, take_string};

/// Safe wrapper around `RPBroadcastHandler`.
pub struct BroadcastHandler {
    pub(crate) ptr: *mut c_void,
}

unsafe impl Send for BroadcastHandler {}
unsafe impl Sync for BroadcastHandler {}

impl BroadcastHandler {
    /// Whether `RPBroadcastHandler` is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_handler_is_supported() }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_borrowed(handler: *mut c_void) -> Result<Self, ReplayKitError> {
        let mut ptr: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_broadcast_handler_retain(handler, &raw mut ptr, &raw mut err) };
        result_from_status(rc, err)?;
        Ok(Self { ptr })
    }

    /// Returns the Objective-C class name for the wrapped handler.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "RPBroadcastHandler".into())
    }

    /// Updates the service-info dictionary sent back to the broadcasting app.
    pub fn update_service_info<T: Serialize>(&self, service_info: &T) -> Result<(), ReplayKitError> {
        let service_info = json_cstring(service_info, "broadcast service info")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_handler_update_service_info(
                self.ptr,
                service_info.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Updates the broadcast URL sent back to the broadcasting app.
    pub fn update_broadcast_url(&self, broadcast_url: &str) -> Result<(), ReplayKitError> {
        let broadcast_url = cstring_from_str(broadcast_url, "broadcast URL")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_handler_update_broadcast_url(
                self.ptr,
                broadcast_url.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }
}

impl Drop for BroadcastHandler {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_object_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for BroadcastHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastHandler")
            .field("class_name", &self.class_name())
            .finish()
    }
}
