use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::take_string;

/// Explicit macOS wrapper for the deprecated iOS-only `RPBroadcastConfiguration`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BroadcastConfiguration;

impl BroadcastConfiguration {
    /// Whether `RPBroadcastConfiguration` is available on this platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_configuration_is_supported() }
    }

    /// Returns the platform-specific unavailability reason.
    pub fn unsupported_reason() -> String {
        let ptr = unsafe { ffi::rk_broadcast_configuration_unavailable_reason() };
        unsafe { take_string(ptr) }
            .unwrap_or_else(|| "RPBroadcastConfiguration is unavailable on macOS".into())
    }

    /// Attempts to construct a broadcast configuration.
    pub fn new() -> Result<Self, ReplayKitError> {
        Err(ReplayKitError::NotSupported(Self::unsupported_reason()))
    }
}
