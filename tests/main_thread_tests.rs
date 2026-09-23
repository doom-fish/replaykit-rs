use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use replaykit::{
    CameraPreviewView, DetailedRecordingEvent, PreviewEvent, PreviewViewController,
    PreviewViewControllerHandle, PreviewViewControllerObserver, ReplayKitError,
    SampleBufferCaptureSession, ScreenRecorder,
};

extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_msgSend();
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
    fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after_source: u8) -> i32;
    static kCFRunLoopDefaultMode: *const c_void;
}

trait AmbiguousIfSend<A> {
    fn check() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

trait AmbiguousIfSync<A> {
    fn check() {}
}
impl<T: ?Sized> AmbiguousIfSync<()> for T {}
impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}

const fn assert_send_sync<T: Send + Sync>() {}

fn send0(receiver: *mut c_void, selector: &CStr) -> *mut c_void {
    let send = unsafe {
        std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void,
        >(objc_msgSend)
    };
    unsafe { send(receiver, sel_registerName(selector.as_ptr())) }
}

fn send1(receiver: *mut c_void, selector: &CStr, argument: *mut c_void) {
    let send = unsafe {
        std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
        >(objc_msgSend)
    };
    unsafe { send(receiver, sel_registerName(selector.as_ptr()), argument) };
}

fn send3(
    receiver: *mut c_void,
    selector: &CStr,
    first: *mut c_void,
    second: *mut c_void,
    third: *mut c_void,
) {
    let send = unsafe {
        std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void),
        >(objc_msgSend)
    };
    unsafe { send(receiver, sel_registerName(selector.as_ptr()), first, second, third) };
}

fn retain_count(object: *mut c_void) -> usize {
    let send = unsafe {
        std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize,
        >(objc_msgSend)
    };
    unsafe { send(object, sel_registerName(c"retainCount".as_ptr())) }
}

fn with_autorelease_pool<R>(body: impl FnOnce() -> R) -> R {
    let pool = unsafe { objc_autoreleasePoolPush() };
    let result = body();
    unsafe { objc_autoreleasePoolPop(pool) };
    result
}

fn run_main_loop_until(mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !done() && Instant::now() < deadline {
        unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 1) };
    }
}

fn shared_recorder_object() -> *mut c_void {
    send0(
        unsafe { objc_getClass(c"RPScreenRecorder".as_ptr()) },
        c"sharedRecorder",
    )
}

fn thread_contracts_hold() {
    <PreviewViewController as AmbiguousIfSend<_>>::check();
    <PreviewViewController as AmbiguousIfSync<_>>::check();
    <CameraPreviewView as AmbiguousIfSend<_>>::check();
    <CameraPreviewView as AmbiguousIfSync<_>>::check();
    <PreviewViewControllerObserver as AmbiguousIfSend<_>>::check();
    <PreviewViewControllerObserver as AmbiguousIfSync<_>>::check();
    assert_send_sync::<PreviewViewControllerHandle>();
    assert_send_sync::<DetailedRecordingEvent>();
    assert_send_sync::<ScreenRecorder>();
    assert_send_sync::<SampleBufferCaptureSession>();
}

fn camera_preview_view_requires_the_main_thread(recorder: &ScreenRecorder) {
    match recorder.camera_preview_view() {
        Ok(Some(view)) => assert!(view.is_hidden().is_ok()),
        Ok(None) => {}
        Err(error) => panic!("camera_preview_view failed on the main thread: {error}"),
    }

    let off_main = thread::spawn(|| {
        let recorder = ScreenRecorder::shared().expect("shared recorder");
        recorder.camera_preview_view().map(|view| view.is_some())
    })
    .join()
    .expect("camera preview thread");
    assert!(matches!(off_main, Err(ReplayKitError::MainThreadRequired(_))));
}

fn deliver_preview_through_the_recorder_delegate(
    recorder: &ScreenRecorder,
    preview: *mut c_void,
) -> PreviewViewControllerHandle {
    let delivered = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&delivered);
    let observer = recorder.observe_detailed(move |event| {
        if let DetailedRecordingEvent::DidStopRecording {
            preview_view_controller: Some(handle),
            error: None,
        } = event
        {
            *sink.lock().unwrap() = Some(handle);
        }
    });

    with_autorelease_pool(|| {
        let recorder_object = shared_recorder_object();
        let delegate = send0(recorder_object, c"delegate");
        assert!(!delegate.is_null());
        send3(
            delegate,
            c"screenRecorder:didStopRecordingWithPreviewViewController:error:",
            recorder_object,
            preview,
            std::ptr::null_mut(),
        );
    });
    drop(observer);

    let handle = delivered.lock().unwrap().take();
    handle.expect("detailed observer received the preview controller")
}

fn preview_controllers_require_the_main_thread(recorder: &ScreenRecorder) {
    let preview = send0(
        unsafe { objc_getClass(c"RPPreviewViewController".as_ptr()) },
        c"new",
    );
    assert!(!preview.is_null());

    let handle = deliver_preview_through_the_recorder_delegate(recorder, preview);
    assert_eq!(handle.class_name(), "RPPreviewViewController");
    let baseline = retain_count(preview);

    with_autorelease_pool(|| {
        let controller = handle.to_controller().expect("controller on the main thread");
        assert_eq!(retain_count(preview), baseline + 1);
        assert_eq!(controller.class_name(), "RPPreviewViewController");
        assert_eq!(controller.is_view_loaded(), Ok(false));

        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let observer = controller
            .observe(move |event| sink.lock().unwrap().push(event))
            .expect("observe on the main thread");
        let delegate = send0(preview, c"previewControllerDelegate");
        assert!(!delegate.is_null());
        send1(delegate, c"previewControllerDidFinish:", preview);
        assert_eq!(*events.lock().unwrap(), vec![PreviewEvent::DidFinish]);

        drop(observer);
        assert!(send0(preview, c"previewControllerDelegate").is_null());
    });
    assert_eq!(retain_count(preview), baseline);

    let off_main = thread::spawn(move || {
        let result = handle.to_controller().map(|controller| controller.class_name());
        drop(handle);
        result
    })
    .join()
    .expect("preview handle thread");
    assert!(matches!(off_main, Err(ReplayKitError::MainThreadRequired(_))));
    assert_eq!(retain_count(preview), baseline);

    run_main_loop_until(|| retain_count(preview) < baseline);
    assert_eq!(retain_count(preview), baseline - 1);

    send0(preview, c"release");
}

fn main() {
    thread_contracts_hold();
    let recorder = ScreenRecorder::shared().expect("shared recorder");
    camera_preview_view_requires_the_main_thread(&recorder);
    preview_controllers_require_the_main_thread(&recorder);
    println!("main_thread_tests: ok");
}
