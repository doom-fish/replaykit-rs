use core::ffi::{c_char, c_void};
use std::ptr;

use serde::Serialize;

use crate::error::{ReplayKitError, ReplayKitFrameworkError};
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, result_from_status, take_string};

/// Safe wrapper around `RPBroadcastSampleHandler`.
pub struct BroadcastSampleHandler {
    ptr: *mut c_void,
}

unsafe impl Send for BroadcastSampleHandler {}
unsafe impl Sync for BroadcastSampleHandler {}

impl BroadcastSampleHandler {
    /// Whether `RPBroadcastSampleHandler` is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_sample_handler_is_supported() }
    }

    /// Constructs a new broadcast sample-handler instance.
    pub fn new() -> Self {
        Self {
            ptr: unsafe { ffi::rk_broadcast_sample_handler_new() },
        }
    }

    /// Returns the Objective-C class name for the wrapped handler.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "RPBroadcastSampleHandler".into())
    }

    /// Updates the service-info dictionary sent back to the broadcasting app.
    pub fn update_service_info<T: Serialize>(&self, service_info: &T) -> Result<(), ReplayKitError> {
        let service_info = json_cstring(service_info, "broadcast service info")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_update_service_info(
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
            ffi::rk_broadcast_sample_handler_update_broadcast_url(
                self.ptr,
                broadcast_url.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Notifies `ReplayKit` that broadcasting started with no setup info.
    pub fn broadcast_started(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_broadcast_started(
                self.ptr,
                ptr::null(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Notifies `ReplayKit` that broadcasting started with JSON-serializable setup info.
    pub fn broadcast_started_with_setup_info<T: Serialize>(
        &self,
        setup_info: &T,
    ) -> Result<(), ReplayKitError> {
        let setup_info = json_cstring(setup_info, "broadcast setup info")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_broadcast_started(
                self.ptr,
                setup_info.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Notifies `ReplayKit` that broadcasting paused.
    pub fn broadcast_paused(&self) {
        unsafe { ffi::rk_broadcast_sample_handler_broadcast_paused(self.ptr) };
    }

    /// Notifies `ReplayKit` that broadcasting resumed.
    pub fn broadcast_resumed(&self) {
        unsafe { ffi::rk_broadcast_sample_handler_broadcast_resumed(self.ptr) };
    }

    /// Notifies `ReplayKit` that broadcasting finished.
    pub fn broadcast_finished(&self) {
        unsafe { ffi::rk_broadcast_sample_handler_broadcast_finished(self.ptr) };
    }

    /// Forwards annotated application info to `ReplayKit`.
    pub fn broadcast_annotated_with_application_info<T: Serialize>(
        &self,
        application_info: &T,
    ) -> Result<(), ReplayKitError> {
        let application_info = json_cstring(application_info, "broadcast application info")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_broadcast_annotated_with_application_info(
                self.ptr,
                application_info.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }

    /// Finishes broadcasting with the supplied framework-style error payload.
    pub fn finish_broadcast_with_error(
        &self,
        error: &ReplayKitFrameworkError,
    ) -> Result<(), ReplayKitError> {
        let domain = cstring_from_str(&error.domain, "broadcast error domain")?;
        let localized_description = cstring_from_str(
            &error.localized_description,
            "broadcast error localized description",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_finish_broadcast_with_error(
                self.ptr,
                domain.as_ptr(),
                error.code,
                localized_description.as_ptr(),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }
}

impl Default for BroadcastSampleHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BroadcastSampleHandler {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_object_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for BroadcastSampleHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastSampleHandler")
            .field("class_name", &self.class_name())
            .finish()
    }
}
