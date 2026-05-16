#![allow(dead_code)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::ffi::status;

pub fn cstring_from_str(value: &str, context: &str) -> Result<CString, ReplayKitError> {
    CString::new(value).map_err(|error| {
        ReplayKitError::InvalidArgument(format!("{context} contains an embedded NUL byte: {error}"))
    })
}

pub fn path_cstring(path: &Path, context: &str) -> Result<CString, ReplayKitError> {
    let path = path.to_str().ok_or_else(|| {
        ReplayKitError::InvalidArgument(format!(
            "{context} path is not valid UTF-8: {}",
            path.display()
        ))
    })?;
    cstring_from_str(path, context)
}

pub fn json_cstring<T: Serialize + ?Sized>(
    value: &T,
    context: &str,
) -> Result<CString, ReplayKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        ReplayKitError::InvalidArgument(format!("failed to encode {context} as JSON: {error}"))
    })?;
    cstring_from_str(&json, context)
}

/// Takes ownership of `ptr` (which was allocated by `strdup` in Swift) and
/// returns the string contents, freeing the pointer.
///
/// # Safety
/// `ptr` must be a valid, NUL-terminated C string allocated via `rk_string_free`-compatible means.
pub unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let string = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::rk_string_free(ptr);
    Some(string)
}

pub unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, ReplayKitError> {
    let json = take_string(ptr).ok_or_else(|| {
        ReplayKitError::InvalidArgument(format!("missing JSON payload for {context}"))
    })?;
    serde_json::from_str(&json).map_err(|error| {
        ReplayKitError::InvalidArgument(format!(
            "failed to parse {context} JSON: {error}; payload={json}"
        ))
    })
}

pub unsafe fn error_from_status(status: i32, err_msg: *mut c_char) -> ReplayKitError {
    crate::error::from_swift(status, err_msg)
}

pub fn result_from_status(status: i32, err_msg: *mut c_char) -> Result<(), ReplayKitError> {
    if status == status::OK {
        Ok(())
    } else {
        Err(unsafe { error_from_status(status, err_msg) })
    }
}
