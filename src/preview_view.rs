use core::ffi::{c_char, c_void};
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

use doom_fish_utils::callback_context::CallbackContext;

use crate::error::ReplayKitError;
use crate::ffi;
use crate::private::{parse_json_ptr, result_from_status, take_string};

type PreviewHandler = Box<dyn Fn(PreviewEvent) + Send + Sync>;

/// Events emitted by `RPPreviewViewControllerDelegate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewEvent {
    /// The preview controller finished.
    DidFinish,
    /// The preview controller finished and reported completed activity types.
    DidFinishWithActivityTypes(Vec<String>),
}

pub struct PreviewViewControllerHandle {
    ptr: *mut c_void,
}

unsafe impl Send for PreviewViewControllerHandle {}
unsafe impl Sync for PreviewViewControllerHandle {}

impl PreviewViewControllerHandle {
    pub(crate) unsafe fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "RPPreviewViewController".into())
    }

    pub fn to_controller(&self) -> Result<PreviewViewController, ReplayKitError> {
        let mut controller: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_preview_view_controller_retain_on_main_thread(
                self.ptr,
                &raw mut controller,
                &raw mut err,
            )
        };
        result_from_status(rc, err)?;
        Ok(PreviewViewController {
            ptr: controller,
            _main_thread_only: PhantomData,
        })
    }
}

impl Drop for PreviewViewControllerHandle {
    fn drop(&mut self) {
        unsafe { ffi::rk_object_release_on_main_thread(self.ptr) };
    }
}

impl std::fmt::Debug for PreviewViewControllerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewControllerHandle")
            .field("class_name", &self.class_name())
            .finish()
    }
}

/// Safe wrapper around `RPPreviewViewController`.
pub struct PreviewViewController {
    ptr: *mut c_void,
    _main_thread_only: PhantomData<Rc<()>>,
}

impl PreviewViewController {
    /// Whether `RPPreviewViewController` is available on the current platform.
    pub fn is_supported_on_current_platform() -> bool {
        unsafe { ffi::rk_preview_view_controller_is_supported() }
    }

    /// Returns the Objective-C class name for the wrapped preview controller.
    pub fn class_name(&self) -> String {
        let ptr = unsafe { ffi::rk_object_class_name(self.ptr) };
        unsafe { take_string(ptr) }.unwrap_or_else(|| "RPPreviewViewController".into())
    }

    /// Returns whether the view hierarchy has been loaded.
    pub fn is_view_loaded(&self) -> Result<bool, ReplayKitError> {
        let mut loaded = false;
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_preview_view_controller_is_view_loaded(self.ptr, &raw mut loaded, &raw mut err)
        };
        result_from_status(rc, err).map(|()| loaded)
    }

    /// Registers a delegate callback for preview controller events.
    pub fn observe<F>(&self, handler: F) -> Result<PreviewViewControllerObserver, ReplayKitError>
    where
        F: Fn(PreviewEvent) + Send + Sync + 'static,
    {
        let handler: PreviewHandler = Box::new(handler);
        let context = CallbackContext::new(handler);
        let mut holder_ptr: *mut c_void = ptr::null_mut();
        let mut err: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            ffi::rk_preview_view_controller_set_delegate(
                self.ptr,
                preview_trampoline,
                context.as_ptr(),
                CallbackContext::<PreviewHandler>::RETAIN,
                CallbackContext::<PreviewHandler>::RELEASE,
                &raw mut holder_ptr,
                &raw mut err,
            )
        };
        result_from_status(rc, err)?;
        Ok(PreviewViewControllerObserver {
            holder_ptr,
            context,
            _main_thread_only: PhantomData,
        })
    }
}

impl Drop for PreviewViewController {
    fn drop(&mut self) {
        unsafe { ffi::rk_object_release_on_main_thread(self.ptr) };
    }
}

impl std::fmt::Debug for PreviewViewController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewController")
            .field("class_name", &self.class_name())
            .finish()
    }
}

unsafe extern "C" fn preview_trampoline(
    context: *mut c_void,
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
    unsafe {
        CallbackContext::<PreviewHandler>::with(
            context,
            "replaykit::preview_view::preview_trampoline",
            |handler| handler(event),
        )
    };
}

/// RAII guard returned by [`PreviewViewController::observe`].
pub struct PreviewViewControllerObserver {
    holder_ptr: *mut c_void,
    context: CallbackContext<PreviewHandler>,
    _main_thread_only: PhantomData<Rc<()>>,
}

impl Drop for PreviewViewControllerObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe { ffi::rk_preview_view_controller_clear_delegate(self.holder_ptr) };
    }
}

impl std::fmt::Debug for PreviewViewControllerObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewViewControllerObserver")
            .finish_non_exhaustive()
    }
}
