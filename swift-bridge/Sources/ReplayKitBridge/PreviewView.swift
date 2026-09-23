import Foundation
import ReplayKit

private let RKPreviewDidFinishEvent: Int32 = 1
private let RKPreviewDidFinishWithActivityTypesEvent: Int32 = 2

final class RKPreviewDelegateHolder: NSObject, RPPreviewViewControllerDelegate {
    typealias Callback = @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafeMutablePointer<CChar>?
    ) -> Void

    let callback: Callback
    let refcon: UnsafeMutableRawPointer?
    let contextRelease: RKContextCallback
    weak var controller: RPPreviewViewController?

    init(
        callback: @escaping Callback,
        refcon: UnsafeMutableRawPointer?,
        contextRetain: RKContextCallback,
        contextRelease: @escaping RKContextCallback,
        controller: RPPreviewViewController
    ) {
        self.callback = callback
        self.refcon = refcon
        self.contextRelease = contextRelease
        self.controller = controller
        // Take a +1 on the Rust CallbackContext for the lifetime of this holder so
        // an in-flight delegate callback can never observe a freed handler.
        contextRetain(refcon)
    }

    deinit {
        contextRelease(refcon)
    }

    func previewControllerDidFinish(_ previewController: RPPreviewViewController) {
        callback(refcon, RKPreviewDidFinishEvent, nil)
    }

    func previewController(
        _ previewController: RPPreviewViewController,
        didFinishWithActivityTypes activityTypes: Set<String>
    ) {
        let payload = (try? rkEncodeJSON(activityTypes.sorted())) ?? "[]"
        callback(refcon, RKPreviewDidFinishWithActivityTypesEvent, rkCString(payload))
    }
}

@_cdecl("rk_preview_view_controller_is_supported")
public func rk_preview_view_controller_is_supported() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_preview_view_controller_retain_on_main_thread")
public func rk_preview_view_controller_retain_on_main_thread(
    _ ptr: UnsafeMutableRawPointer,
    _ outController: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outController?.pointee = nil
    guard rkRequireMainThread("RPPreviewViewController", outError) else {
        return RK_MAIN_THREAD_REQUIRED
    }
    outController?.pointee = rk_object_retain(ptr)
    return RK_OK
}

@_cdecl("rk_preview_view_controller_is_view_loaded")
public func rk_preview_view_controller_is_view_loaded(
    _ ptr: UnsafeMutableRawPointer,
    _ outLoaded: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard rkRequireMainThread("RPPreviewViewController.isViewLoaded", outError) else {
        return RK_MAIN_THREAD_REQUIRED
    }
    outLoaded?.pointee = rk_borrow(ptr, as: RPPreviewViewController.self).isViewLoaded
    return RK_OK
}

@_cdecl("rk_preview_view_controller_set_delegate")
public func rk_preview_view_controller_set_delegate(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafeMutablePointer<CChar>?
    ) -> Void,
    _ refcon: UnsafeMutableRawPointer?,
    _ contextRetain: RKContextCallback,
    _ contextRelease: RKContextCallback,
    _ outHolder: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outHolder?.pointee = nil
    guard rkRequireMainThread("RPPreviewViewController.previewControllerDelegate", outError) else {
        return RK_MAIN_THREAD_REQUIRED
    }
    let controller = rk_borrow(controllerPtr, as: RPPreviewViewController.self)
    let holder = RKPreviewDelegateHolder(
        callback: callback,
        refcon: refcon,
        contextRetain: contextRetain,
        contextRelease: contextRelease,
        controller: controller
    )
    controller.previewControllerDelegate = holder
    outHolder?.pointee = rk_retain(holder)
    return RK_OK
}

@_cdecl("rk_preview_view_controller_clear_delegate")
public func rk_preview_view_controller_clear_delegate(_ holderPtr: UnsafeMutableRawPointer) {
    let holder = Unmanaged<RKPreviewDelegateHolder>.fromOpaque(holderPtr).takeRetainedValue()
    rkOnMainThread {
        if let controller = holder.controller, controller.previewControllerDelegate === holder {
            controller.previewControllerDelegate = nil
        }
    }
}
