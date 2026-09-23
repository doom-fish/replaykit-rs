use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::take_string;

/// Unsupported placeholder for `RPBroadcastSampleHandler`, which only works as the principal class of a broadcast upload extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastSampleHandler {}

impl BroadcastSampleHandler {
    /// Whether this crate can drive `RPBroadcastSampleHandler` on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_sample_handler_is_supported() }
    }

    pub fn unsupported_reason() -> String {
        let ptr = unsafe { ffi::rk_broadcast_sample_handler_unavailable_reason() };
        unsafe { take_string(ptr) }
            .unwrap_or_else(|| "RPBroadcastSampleHandler is not supported by replaykit-rs".into())
    }

    /// Always fails with [`ReplayKitError::NotSupported`].
    pub fn new() -> Result<Self, ReplayKitError> {
        Err(ReplayKitError::NotSupported(Self::unsupported_reason()))
    }
}
