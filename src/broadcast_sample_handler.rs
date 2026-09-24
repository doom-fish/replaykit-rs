use core::ffi::{c_char, c_void};
use std::ptr;

use crate::broadcast_handler::BroadcastHandler;
use crate::error::{ReplayKitError, ReplayKitFrameworkError};
use crate::ffi;
use crate::private::{cstring_from_str, result_from_status};

/// Safe wrapper around a broadcast extension's own `RPBroadcastSampleHandler`.
#[derive(Debug)]
pub struct BroadcastSampleHandler {
    handler: BroadcastHandler,
}

impl BroadcastSampleHandler {
    /// Whether this crate can drive `RPBroadcastSampleHandler` on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_sample_handler_is_supported() }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_borrowed(sample_handler: *mut c_void) -> Result<Self, ReplayKitError> {
        let mut ptr: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_sample_handler_retain(sample_handler, &raw mut ptr, &raw mut err)
        };
        result_from_status(rc, err)?;
        Ok(Self {
            handler: BroadcastHandler { ptr },
        })
    }

    pub const fn as_handler(&self) -> &BroadcastHandler {
        &self.handler
    }

    pub fn finish_broadcast_with_error(
        &self,
        error: &ReplayKitFrameworkError,
    ) -> Result<(), ReplayKitError> {
        let domain = cstring_from_str(&error.domain, "broadcast error domain")?;
        let localized_description = cstring_from_str(
            &error.localized_description,
            "broadcast error localized description",
        )?;
        unsafe {
            ffi::rk_broadcast_sample_handler_finish_broadcast_with_error(
                self.handler.ptr,
                domain.as_ptr(),
                error.code,
                localized_description.as_ptr(),
            );
        }
        Ok(())
    }
}
