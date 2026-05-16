use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::take_string;

/// Explicit macOS wrapper for the iOS-only `RPSystemBroadcastPickerView`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemBroadcastPickerView;

impl SystemBroadcastPickerView {
    /// Whether the system broadcast picker view is available on this platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_system_broadcast_picker_view_is_supported() }
    }

    /// Returns the platform-specific unavailability reason.
    pub fn unsupported_reason() -> String {
        let ptr = unsafe { ffi::rk_system_broadcast_picker_view_unavailable_reason() };
        unsafe { take_string(ptr) }
            .unwrap_or_else(|| "RPSystemBroadcastPickerView is unavailable on macOS".into())
    }

    /// Attempts to construct a system broadcast picker view.
    pub fn new(
        _preferred_extension: Option<&str>,
        _shows_microphone_button: bool,
    ) -> Result<Self, ReplayKitError> {
        Err(ReplayKitError::NotSupported(Self::unsupported_reason()))
    }
}
