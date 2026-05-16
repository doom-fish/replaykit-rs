use core::ffi::c_char;
use std::fmt;

use serde::Deserialize;

use crate::ffi::status;
use crate::private::take_string;

/// Errors returned by `ReplayKit` bridge operations.
#[derive(Debug, Clone)]
pub enum ReplayKitError {
    /// A bad argument was supplied.
    InvalidArgument(String),
    /// The operation timed out.
    TimedOut(String),
    /// The feature is not supported on this platform or OS version.
    NotSupported(String),
    /// An underlying `ReplayKit` / Objective-C framework error.
    Framework(ReplayKitFrameworkError),
    /// An error with no further classification.
    Unknown(String),
}

impl fmt::Display for ReplayKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(msg)
            | Self::TimedOut(msg)
            | Self::NotSupported(msg)
            | Self::Unknown(msg) => f.write_str(msg),
            Self::Framework(err) => write!(
                f,
                "{} (domain={}, code={})",
                err.localized_description, err.domain, err.code
            ),
        }
    }
}

impl std::error::Error for ReplayKitError {}

/// An Objective-C `NSError`-style error from the `ReplayKit` framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayKitFrameworkError {
    /// The `NSError` domain.
    pub domain: String,
    /// The `NSError` code.
    pub code: i64,
    /// The human-readable description.
    pub localized_description: String,
}

#[derive(Deserialize)]
struct FrameworkErrorPayload {
    domain: String,
    code: i64,
    #[serde(rename = "localizedDescription")]
    localized_description: String,
}

/// Build a `ReplayKitError` from a raw Swift status code and optional error
/// message pointer.
///
/// # Safety
/// `err_msg` must be NULL or a valid pointer to a C string owned by the Rust
/// side (it will be freed via `rk_string_free`).
pub(crate) unsafe fn from_swift(status: i32, err_msg: *mut c_char) -> ReplayKitError {
    let msg = take_string(err_msg).unwrap_or_default();

    match status {
        s if s == status::INVALID_ARGUMENT => ReplayKitError::InvalidArgument(msg),
        s if s == status::TIMED_OUT => ReplayKitError::TimedOut(msg),
        s if s == status::NOT_SUPPORTED => ReplayKitError::NotSupported(msg),
        s if s == status::FRAMEWORK_ERROR => {
            // msg may be JSON — try to deserialize; fall back to plain string
            if let Ok(payload) = serde_json::from_str::<FrameworkErrorPayload>(&msg) {
                ReplayKitError::Framework(ReplayKitFrameworkError {
                    domain: payload.domain,
                    code: payload.code,
                    localized_description: payload.localized_description,
                })
            } else {
                ReplayKitError::Framework(ReplayKitFrameworkError {
                    domain: "RPRecordingErrorDomain".into(),
                    code: i64::from(status),
                    localized_description: msg,
                })
            }
        }
        _ => ReplayKitError::Unknown(if msg.is_empty() {
            format!("unknown error (status={status})")
        } else {
            msg
        }),
    }
}
