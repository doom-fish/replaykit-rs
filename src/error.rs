use core::ffi::c_char;
use std::fmt;

use serde::Deserialize;

use crate::ffi::status;
use crate::private::take_string;

/// Objective-C domain used for `ReplayKit` recording and broadcast errors.
pub const RP_RECORDING_ERROR_DOMAIN: &str = "RPRecordingErrorDomain";
/// `ScreenCaptureKit` error domain re-exported by `ReplayKit` headers.
pub const SC_STREAM_ERROR_DOMAIN: &str = "SCStreamErrorDomain";

/// Errors returned by the `ReplayKit` bridge operations.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ReplayKitFrameworkError {
    /// Maps the framework error code into a typed `RPRecordingErrorCode` when possible.
    pub const fn recording_code(&self) -> Option<RecordingErrorCode> {
        RecordingErrorCode::from_i64(self.code)
    }
}

/// Known `RPRecordingErrorCode` values from `RPError.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum RecordingErrorCode {
    Unknown = -5800,
    UserDeclined = -5801,
    Disabled = -5802,
    FailedToStart = -5803,
    Failed = -5804,
    InsufficientStorage = -5805,
    Interrupted = -5806,
    ContentResize = -5807,
    BroadcastInvalidSession = -5808,
    SystemDormancy = -5809,
    Entitlements = -5810,
    ActivePhoneCall = -5811,
    FailedToSave = -5812,
    CarPlay = -5813,
    FailedApplicationConnectionInvalid = -5814,
    FailedApplicationConnectionInterrupted = -5815,
    FailedNoMatchingApplicationContext = -5816,
    FailedMediaServicesFailure = -5817,
    VideoMixingFailure = -5818,
    BroadcastSetupFailed = -5819,
    FailedToObtainUrl = -5820,
    FailedIncorrectTimeStamps = -5821,
    FailedToProcessFirstSample = -5822,
    FailedAssetWriterFailedToSave = -5823,
    FailedNoAssetWriter = -5824,
    FailedAssetWriterInWrongState = -5825,
    FailedAssetWriterExportFailed = -5826,
    FailedToRemoveFile = -5827,
    FailedAssetWriterExportCanceled = -5828,
    AttemptToStopNonRecording = -5829,
    AttemptToStartInRecordingState = -5830,
    PhotoFailure = -5831,
    RecordingInvalidSession = -5832,
    FailedToStartCaptureStack = -5833,
    InvalidParameter = -5834,
    FilePermissions = -5835,
    ExportClipToUrlInProgress = -5836,
    Successful = 0,
}

impl RecordingErrorCode {
    /// Converts a raw `RPRecordingErrorCode` into the typed enum.
    pub const fn from_i64(code: i64) -> Option<Self> {
        Some(match code {
            -5800 => Self::Unknown,
            -5801 => Self::UserDeclined,
            -5802 => Self::Disabled,
            -5803 => Self::FailedToStart,
            -5804 => Self::Failed,
            -5805 => Self::InsufficientStorage,
            -5806 => Self::Interrupted,
            -5807 => Self::ContentResize,
            -5808 => Self::BroadcastInvalidSession,
            -5809 => Self::SystemDormancy,
            -5810 => Self::Entitlements,
            -5811 => Self::ActivePhoneCall,
            -5812 => Self::FailedToSave,
            -5813 => Self::CarPlay,
            -5814 => Self::FailedApplicationConnectionInvalid,
            -5815 => Self::FailedApplicationConnectionInterrupted,
            -5816 => Self::FailedNoMatchingApplicationContext,
            -5817 => Self::FailedMediaServicesFailure,
            -5818 => Self::VideoMixingFailure,
            -5819 => Self::BroadcastSetupFailed,
            -5820 => Self::FailedToObtainUrl,
            -5821 => Self::FailedIncorrectTimeStamps,
            -5822 => Self::FailedToProcessFirstSample,
            -5823 => Self::FailedAssetWriterFailedToSave,
            -5824 => Self::FailedNoAssetWriter,
            -5825 => Self::FailedAssetWriterInWrongState,
            -5826 => Self::FailedAssetWriterExportFailed,
            -5827 => Self::FailedToRemoveFile,
            -5828 => Self::FailedAssetWriterExportCanceled,
            -5829 => Self::AttemptToStopNonRecording,
            -5830 => Self::AttemptToStartInRecordingState,
            -5831 => Self::PhotoFailure,
            -5832 => Self::RecordingInvalidSession,
            -5833 => Self::FailedToStartCaptureStack,
            -5834 => Self::InvalidParameter,
            -5835 => Self::FilePermissions,
            -5836 => Self::ExportClipToUrlInProgress,
            0 => Self::Successful,
            _ => return None,
        })
    }
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
        s if s == status::FRAMEWORK_ERROR => parse_framework_error(&msg),
        _ => ReplayKitError::Unknown(if msg.is_empty() {
            format!("unknown error (status={status})")
        } else {
            msg
        }),
    }
}

pub(crate) fn from_message(message: &str) -> ReplayKitError {
    parse_framework_error(message)
}

fn parse_framework_error(message: &str) -> ReplayKitError {
    if let Ok(payload) = serde_json::from_str::<FrameworkErrorPayload>(message) {
        ReplayKitError::Framework(ReplayKitFrameworkError {
            domain: payload.domain,
            code: payload.code,
            localized_description: payload.localized_description,
        })
    } else {
        ReplayKitError::Unknown(message.to_owned())
    }
}
