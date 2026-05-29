use core::ffi::{c_char, c_void};

use crate::ffi;
use crate::private::{
    context_release_cb, context_retain_cb, parse_json_ptr, take_string, CallbackBox,
};

type PreviewHandler = dyn Fn(PreviewEvent) + Send + 'static;

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
        let boxed: Box<PreviewHandler> = Box::new(handler);
        let context = CallbackBox::into_raw(boxed);
        let refcon = context.cast::<c_void>();
        let holder_ptr = unsafe {
            ffi::rk_preview_view_controller_set_delegate(
                self.ptr,
                preview_trampoline,
                refcon,
                context_retain_cb::<PreviewHandler>,
                context_release_cb::<PreviewHandler>,
            )
        };
        PreviewViewControllerObserver {
            controller_ptr: self.ptr,
            holder_ptr,
            context,
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
    let handler = unsafe { CallbackBox::<PreviewHandler>::handler(refcon.cast()) };
    doom_fish_utils::panic_safe::catch_user_panic(
        "replaykit::preview_view::preview_trampoline",
        || handler(event),
    );
}

/// RAII guard returned by [`PreviewViewController::observe`].
pub struct PreviewViewControllerObserver {
    controller_ptr: *mut c_void,
    holder_ptr: *mut c_void,
    context: *mut CallbackBox<PreviewHandler>,
}

unsafe impl Send for PreviewViewControllerObserver {}
unsafe impl Sync for PreviewViewControllerObserver {}

impl Drop for PreviewViewControllerObserver {
    fn drop(&mut self) {
        unsafe {
            ffi::rk_preview_view_controller_clear_delegate(self.controller_ptr, self.holder_ptr);
            // Release this guard's reference. The Swift holder dropped its own
            // reference in `deinit` (triggered by `clear_delegate`), so the
            // `CallbackBox` is freed only once no callback can still run.
            CallbackBox::release(self.context);
        }
    }
}

impl std::fmt::Debug for PreviewViewControllerObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewControllerObserver")
            .finish_non_exhaustive()
    }
}
