use core::ffi::{c_char, c_void};
use std::ffi::CString;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr, result_from_status, take_string};

/// `ReplayKit` dictionary key containing the broadcasting app bundle identifier.
pub const RP_APPLICATION_INFO_BUNDLE_IDENTIFIER_KEY: &str = "RPApplicationInfoBundleIdentifier";

/// Application metadata loaded from an `NSExtensionContext` during broadcast setup.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BroadcastingApplicationInfo {
    /// The broadcasting app bundle identifier.
    #[serde(rename = "bundleIdentifier")]
    pub bundle_identifier: String,
    /// The app display name shown by `ReplayKit`.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// Whether `ReplayKit` supplied an app icon.
    #[serde(rename = "hasAppIcon")]
    pub has_app_icon: bool,
    /// Objective-C class name of the icon object when one was supplied.
    #[serde(rename = "appIconClassName")]
    pub app_icon_class_name: Option<String>,
}

/// Safe wrapper around `NSExtensionContext` broadcast-extension helpers.
pub struct BroadcastExtensionContext {
    ptr: *mut c_void,
}

unsafe impl Send for BroadcastExtensionContext {}
unsafe impl Sync for BroadcastExtensionContext {}

impl BroadcastExtensionContext {
    /// Whether the `ReplayKit` broadcast-extension category is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_extension_context_is_supported() }
    }

    /// Constructs a standalone extension context.
    ///
    /// In a real broadcast extension, `ReplayKit` supplies the context instance.
    pub fn new() -> Self {
        Self {
            ptr: unsafe { ffi::rk_broadcast_extension_context_new() },
        }
    }

    /// Returns the Objective-C class name for the wrapped context.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "NSExtensionContext".into())
    }

    /// Loads information about the broadcasting app.
    ///
    /// `ReplayKit` only resolves this information for an extension-owned context.
    pub fn load_broadcasting_application_info(
        &self,
    ) -> Result<BroadcastingApplicationInfo, ReplayKitError> {
        let mut json: *mut c_char = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_extension_context_load_application_info_json(
                self.ptr,
                &raw mut json,
                &raw mut err,
            )
        };
        if rc == crate::ffi::status::OK {
            unsafe { parse_json_ptr(json, "broadcasting application info") }
        } else {
            if !json.is_null() {
                unsafe { ffi::rk_string_free(json) };
            }
            Err(unsafe { crate::private::error_from_status(rc, err) })
        }
    }

    /// Completes the extension request with a broadcast URL and no setup info.
    pub fn complete_request_with_broadcast_url(
        &self,
        broadcast_url: &str,
    ) -> Result<(), ReplayKitError> {
        self.complete_request_inner(broadcast_url, None)
    }

    /// Completes the extension request with a broadcast URL and JSON-serializable setup info.
    pub fn complete_request_with_broadcast_url_and_setup_info<T: Serialize>(
        &self,
        broadcast_url: &str,
        setup_info: &T,
    ) -> Result<(), ReplayKitError> {
        let setup_info = json_cstring(setup_info, "broadcast setup info")?;
        self.complete_request_inner(broadcast_url, Some(&setup_info))
    }

    fn complete_request_inner(
        &self,
        broadcast_url: &str,
        setup_info_json: Option<&CString>,
    ) -> Result<(), ReplayKitError> {
        let broadcast_url = cstring_from_str(broadcast_url, "broadcast URL")?;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_broadcast_extension_context_complete_request_with_broadcast_url(
                self.ptr,
                broadcast_url.as_ptr(),
                setup_info_json.map_or(ptr::null(), |json| json.as_ptr()),
                &raw mut err,
            )
        };
        result_from_status(rc, err)
    }
}

impl Default for BroadcastExtensionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BroadcastExtensionContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_object_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for BroadcastExtensionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastExtensionContext")
            .field("class_name", &self.class_name())
            .finish()
    }
}
