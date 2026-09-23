#![allow(dead_code)]

use core::ffi::{c_char, c_void};
use std::ffi::CString;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    doom_fish_utils::ffi_string::take_owned_cstring_c(ptr, |p| ffi::rk_string_free(p))
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

/// Reference-counted heap allocation handed across the Swift FFI boundary.
///
/// The Rust owner (an RAII observer guard or capture session) holds the
/// initial reference. Each Swift delegate/handler object takes an additional
/// reference in its `init` (via [`context_retain_cb`]) and drops it in its
/// `deinit` (via [`context_release_cb`]). The inner `handler` — and therefore
/// the heap allocation — is freed only once every holder has released its
/// reference, so an in-flight callback dispatched on another queue can never
/// observe a freed handler. This mirrors the `StreamContext` refcount pattern
/// used in screencapturekit-rs.
pub struct CallbackBox<T: ?Sized> {
    handler: Box<T>,
    ref_count: AtomicUsize,
}

impl<T: ?Sized> CallbackBox<T> {
    /// Allocates a new context with a reference count of 1 and returns the
    /// raw pointer to hand across FFI.
    pub fn into_raw(handler: Box<T>) -> *mut Self {
        Box::into_raw(Box::new(Self {
            handler,
            ref_count: AtomicUsize::new(1),
        }))
    }

    /// Borrows the inner handler.
    ///
    /// # Safety
    /// `ptr` must point to a live `CallbackBox<T>` whose reference count is
    /// held for the duration of the returned borrow.
    pub unsafe fn handler<'a>(ptr: *mut Self) -> &'a T {
        &(*ptr).handler
    }

    /// Increments the reference count.
    ///
    /// # Safety
    /// `ptr` must be null or point to a live `CallbackBox<T>`.
    pub unsafe fn retain(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }
        (*ptr).ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the reference count, freeing the allocation when it reaches
    /// zero.
    ///
    /// # Safety
    /// `ptr` must be null or point to a live `CallbackBox<T>`. After the call
    /// `ptr` must not be used if the allocation was freed.
    pub unsafe fn release(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }
        if (*ptr).ref_count.fetch_sub(1, Ordering::Release) == 1 {
            // Acquire fence pairs with the Release stores from other threads'
            // `fetch_sub` calls — the canonical Arc-style refcount drop.
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(ptr));
        }
    }
}

/// C trampoline handed to Swift so a delegate/handler object can take a +1
/// reference on the [`CallbackBox`] for the duration of its own lifetime.
pub extern "C" fn context_retain_cb<T: ?Sized>(context: *mut c_void) {
    unsafe { CallbackBox::<T>::retain(context.cast::<CallbackBox<T>>()) };
}

/// C trampoline handed to Swift, invoked from a delegate/handler object's
/// `deinit` to drop the reference taken in [`context_retain_cb`].
pub extern "C" fn context_release_cb<T: ?Sized>(context: *mut c_void) {
    unsafe { CallbackBox::<T>::release(context.cast::<CallbackBox<T>>()) };
}

#[cfg(test)]
pub fn recorder_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
