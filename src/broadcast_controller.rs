use core::ffi::{c_char, c_void};
use std::ptr;

use serde_json::Value;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{parse_json_ptr, result_from_status, take_string};

/// Events emitted by `RPBroadcastControllerDelegate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BroadcastControllerEvent {
    /// The broadcast finished, optionally with a framework error.
    DidFinish { error: Option<ReplayKitError> },
    /// The broadcast service published updated service info.
    DidUpdateServiceInfo(Value),
    /// The broadcast URL changed.
    DidUpdateBroadcastUrl(String),
}

/// A safe wrapper around `RPBroadcastController` (macOS 11+).
///
/// Obtained from the completion callback of
/// [`crate::broadcast_activity_view_controller::BroadcastActivityControllerHandle::show`].
pub struct BroadcastController {
    pub(crate) ptr: *mut c_void,
}

unsafe impl Send for BroadcastController {}
unsafe impl Sync for BroadcastController {}

impl BroadcastController {
    /// Whether `RPBroadcastController` is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_broadcast_controller_is_supported() }
    }

    pub(crate) const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Whether a broadcast is currently active.
    pub fn is_broadcasting(&self) -> bool {
        unsafe { ffi::rk_broadcast_controller_is_broadcasting(self.ptr) }
    }

    /// Whether the broadcast is paused.
    pub fn is_paused(&self) -> bool {
        unsafe { ffi::rk_broadcast_controller_is_paused(self.ptr) }
    }

    /// The URL where the broadcast can be watched.
    pub fn broadcast_url(&self) -> String {
        let ptr = unsafe { ffi::rk_broadcast_controller_broadcast_url(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_default()
    }

    /// Returns the service-info dictionary as JSON when one is available.
    pub fn service_info(&self) -> Result<Option<Value>, ReplayKitError> {
        let ptr = unsafe { ffi::rk_broadcast_controller_service_info_json(self.ptr) };
        if ptr.is_null() {
            Ok(None)
        } else {
            unsafe { parse_json_ptr(ptr, "broadcast service info") }.map(Some)
        }
    }

    /// Registers a delegate callback for broadcast-controller events.
    pub fn observe<F>(&self, handler: F) -> BroadcastControllerObserver
    where
        F: Fn(BroadcastControllerEvent) + Send + 'static,
    {
        let boxed: Box<dyn Fn(BroadcastControllerEvent) + Send + 'static> = Box::new(handler);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();
        let holder_ptr = unsafe {
            ffi::rk_broadcast_controller_set_delegate(self.ptr, delegate_trampoline, refcon)
        };
        BroadcastControllerObserver {
            controller_ptr: self.ptr,
            holder_ptr,
            refcon,
        }
    }

    /// Starts the broadcast.
    pub fn start(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_broadcast_controller_start(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Finishes the broadcast.
    pub fn finish(&self) -> Result<(), ReplayKitError> {
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe { ffi::rk_broadcast_controller_finish(self.ptr, &raw mut err) };
        result_from_status(rc, err)
    }

    /// Pauses the broadcast.
    pub fn pause(&self) {
        unsafe { ffi::rk_broadcast_controller_pause(self.ptr) };
    }

    /// Resumes a paused broadcast.
    pub fn resume(&self) {
        unsafe { ffi::rk_broadcast_controller_resume(self.ptr) };
    }
}

impl Drop for BroadcastController {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_broadcast_controller_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for BroadcastController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastController")
            .field("is_broadcasting", &self.is_broadcasting())
            .field("is_paused", &self.is_paused())
            .finish()
    }
}

unsafe extern "C" fn delegate_trampoline(
    refcon: *mut c_void,
    event_kind: i32,
    payload: *mut c_char,
) {
    let handler = &*(refcon.cast::<Box<dyn Fn(BroadcastControllerEvent) + Send + 'static>>());
    let event = match event_kind {
        1 => {
            let error =
                unsafe { take_string(payload) }.map(|message| crate::error::from_message(&message));
            BroadcastControllerEvent::DidFinish { error }
        }
        2 => unsafe {
            parse_json_ptr::<Value>(payload, "broadcast service info event").map_or_else(
                |error| BroadcastControllerEvent::DidFinish { error: Some(error) },
                BroadcastControllerEvent::DidUpdateServiceInfo,
            )
        },
        3 => BroadcastControllerEvent::DidUpdateBroadcastUrl(
            unsafe { take_string(payload) }.unwrap_or_default(),
        ),
        _ => BroadcastControllerEvent::DidFinish {
            error: Some(ReplayKitError::Unknown(format!(
                "unknown broadcast controller event kind: {event_kind}"
            ))),
        },
    };
    handler(event);
}

/// RAII guard returned by [`BroadcastController::observe`].
pub struct BroadcastControllerObserver {
    controller_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    refcon: *mut c_void,
}

unsafe impl Send for BroadcastControllerObserver {}
unsafe impl Sync for BroadcastControllerObserver {}

impl Drop for BroadcastControllerObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_broadcast_controller_clear_delegate(self.controller_ptr, self.holder_ptr);
            drop(Box::from_raw(
                self.refcon
                    .cast::<Box<dyn Fn(BroadcastControllerEvent) + Send + 'static>>(),
            ));
        }
    }
}

impl std::fmt::Debug for BroadcastControllerObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastControllerObserver")
            .finish_non_exhaustive()
    }
}
