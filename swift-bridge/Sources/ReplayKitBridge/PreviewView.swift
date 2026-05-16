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

    init(callback: @escaping Callback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
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

@_cdecl("rk_preview_view_controller_is_view_loaded")
public func rk_preview_view_controller_is_view_loaded(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPPreviewViewController.self).isViewLoaded
}

@_cdecl("rk_preview_view_controller_set_delegate")
public func rk_preview_view_controller_set_delegate(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafeMutablePointer<CChar>?
    ) -> Void,
    _ refcon: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer {
    let controller = rk_borrow(controllerPtr, as: RPPreviewViewController.self)
    let holder = RKPreviewDelegateHolder(callback: callback, refcon: refcon)
    controller.previewControllerDelegate = holder
    return rk_retain(holder)
}

@_cdecl("rk_preview_view_controller_clear_delegate")
public func rk_preview_view_controller_clear_delegate(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ holderPtr: UnsafeMutableRawPointer
) {
    let controller = rk_borrow(controllerPtr, as: RPPreviewViewController.self)
    controller.previewControllerDelegate = nil
    rk_release(holderPtr)
}
