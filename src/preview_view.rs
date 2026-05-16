use core::ffi::{c_char, c_void};

use crate::ffi;
use crate::private::{parse_json_ptr, take_string};

/// Events emitted by `RPPreviewViewControllerDelegate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewEvent {
    /// The preview controller finished.
    DidFinish,
    /// The preview controller finished and reported completed activity types.
    DidFinishWithActivityTypes(Vec<String>),
}

/// Safe wrapper around `RPPreviewViewController`.
pub struct PreviewViewController {
    pub(crate) ptr: *mut c_void,
}

unsafe impl Send for PreviewViewController {}
unsafe impl Sync for PreviewViewController {}

impl PreviewViewController {
    /// Whether `RPPreviewViewController` is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_preview_view_controller_is_supported() }
    }

    pub(crate) const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns the Objective-C class name for the wrapped preview controller.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "RPPreviewViewController".into())
    }

    /// Returns whether the view hierarchy has been loaded.
    pub fn is_view_loaded(&self) -> bool {
        unsafe { ffi::rk_preview_view_controller_is_view_loaded(self.ptr) }
    }

    /// Registers a delegate callback for preview controller events.
    pub fn observe<F>(&self, handler: F) -> PreviewViewControllerObserver
    where
        F: Fn(PreviewEvent) + Send + 'static,
    {
        let boxed: Box<dyn Fn(PreviewEvent) + Send + 'static> = Box::new(handler);
        let refcon = Box::into_raw(Box::new(boxed)).cast::<c_void>();
        let holder_ptr = unsafe {
            ffi::rk_preview_view_controller_set_delegate(self.ptr, preview_trampoline, refcon)
        };
        PreviewViewControllerObserver {
            controller_ptr: self.ptr,
            holder_ptr,
            refcon,
        }
    }
}

impl Drop for PreviewViewController {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::rk_object_release(self.ptr) };
        }
    }
}

impl std::fmt::Debug for PreviewViewController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewController")
            .field("class_name", &self.class_name())
            .field("is_view_loaded", &self.is_view_loaded())
            .finish()
    }
}

unsafe extern "C" fn preview_trampoline(
    refcon: *mut c_void,
    event_kind: i32,
    activity_types_json: *mut c_char,
) {
    let handler = &*(refcon.cast::<Box<dyn Fn(PreviewEvent) + Send + 'static>>());
    let event = match event_kind {
        2 => unsafe {
            parse_json_ptr::<Vec<String>>(activity_types_json, "preview activity types")
                .map_or_else(
                    |error| PreviewEvent::DidFinishWithActivityTypes(vec![error.to_string()]),
                    PreviewEvent::DidFinishWithActivityTypes,
                )
        },
        _ => PreviewEvent::DidFinish,
    };
    handler(event);
}

/// RAII guard returned by [`PreviewViewController::observe`].
pub struct PreviewViewControllerObserver {
    controller_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    refcon: *mut c_void,
}

unsafe impl Send for PreviewViewControllerObserver {}
unsafe impl Sync for PreviewViewControllerObserver {}

impl Drop for PreviewViewControllerObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_preview_view_controller_clear_delegate(self.controller_ptr, self.holder_ptr);
            drop(Box::from_raw(
                self.refcon
                    .cast::<Box<dyn Fn(PreviewEvent) + Send + 'static>>(),
            ));
        }
    }
}

impl std::fmt::Debug for PreviewViewControllerObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewControllerObserver")
            .finish_non_exhaustive()
    }
}
