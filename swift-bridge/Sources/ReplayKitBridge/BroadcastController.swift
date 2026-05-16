import Foundation
import ReplayKit

private let RKBroadcastControllerDidFinishEvent: Int32 = 1
private let RKBroadcastControllerDidUpdateServiceInfoEvent: Int32 = 2
private let RKBroadcastControllerDidUpdateBroadcastUrlEvent: Int32 = 3

final class RKBroadcastControllerDelegateHolder: NSObject, RPBroadcastControllerDelegate {
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

    func broadcastController(_ broadcastController: RPBroadcastController, didFinishWithError error: Error?) {
        callback(refcon, RKBroadcastControllerDidFinishEvent, error.flatMap(rkOwnedErrorCString))
    }

    func broadcastController(
        _ broadcastController: RPBroadcastController,
        didUpdateServiceInfo serviceInfo: [String: any NSCoding & NSObjectProtocol]
    ) {
        let payload = rkServiceInfoJSON(serviceInfo) ?? "{}"
        callback(refcon, RKBroadcastControllerDidUpdateServiceInfoEvent, rkCString(payload))
    }

    func broadcastController(
        _ broadcastController: RPBroadcastController,
        didUpdateBroadcast broadcastURL: URL
    ) {
        callback(
            refcon,
            RKBroadcastControllerDidUpdateBroadcastUrlEvent,
            rkCString(broadcastURL.absoluteString)
        )
    }
}

@_cdecl("rk_broadcast_controller_is_supported")
public func rk_broadcast_controller_is_supported() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_broadcast_controller_release")
public func rk_broadcast_controller_release(_ ptr: UnsafeMutableRawPointer) {
    rk_release(ptr)
}

@_cdecl("rk_broadcast_controller_is_broadcasting")
public func rk_broadcast_controller_is_broadcasting(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPBroadcastController.self).isBroadcasting
}

@_cdecl("rk_broadcast_controller_is_paused")
public func rk_broadcast_controller_is_paused(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPBroadcastController.self).isPaused
}

@_cdecl("rk_broadcast_controller_broadcast_url")
public func rk_broadcast_controller_broadcast_url(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let url = rk_borrow(ptr, as: RPBroadcastController.self).broadcastURL.absoluteString
    return rkCString(url)
}

@_cdecl("rk_broadcast_controller_service_info_json")
public func rk_broadcast_controller_service_info_json(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let controller = rk_borrow(ptr, as: RPBroadcastController.self)
    guard let json = rkServiceInfoJSON(controller.serviceInfo) else { return nil }
    return rkCString(json)
}

@_cdecl("rk_broadcast_controller_start")
public func rk_broadcast_controller_start(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let controller = rk_borrow(ptr, as: RPBroadcastController.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                controller.startBroadcast { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume()
                    }
                }
            }
        },
        onSuccess: { _ in },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_broadcast_controller_finish")
public func rk_broadcast_controller_finish(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let controller = rk_borrow(ptr, as: RPBroadcastController.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                controller.finishBroadcast { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume()
                    }
                }
            }
        },
        onSuccess: { _ in },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_broadcast_controller_pause")
public func rk_broadcast_controller_pause(_ ptr: UnsafeMutableRawPointer) {
    rk_borrow(ptr, as: RPBroadcastController.self).pauseBroadcast()
}

@_cdecl("rk_broadcast_controller_resume")
public func rk_broadcast_controller_resume(_ ptr: UnsafeMutableRawPointer) {
    rk_borrow(ptr, as: RPBroadcastController.self).resumeBroadcast()
}

@_cdecl("rk_broadcast_controller_set_delegate")
public func rk_broadcast_controller_set_delegate(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafeMutablePointer<CChar>?
    ) -> Void,
    _ refcon: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer {
    let controller = rk_borrow(controllerPtr, as: RPBroadcastController.self)
    let holder = RKBroadcastControllerDelegateHolder(callback: callback, refcon: refcon)
    controller.delegate = holder
    return rk_retain(holder)
}

@_cdecl("rk_broadcast_controller_clear_delegate")
public func rk_broadcast_controller_clear_delegate(
    _ controllerPtr: UnsafeMutableRawPointer,
    _ holderPtr: UnsafeMutableRawPointer
) {
    let controller = rk_borrow(controllerPtr, as: RPBroadcastController.self)
    controller.delegate = nil
    rk_release(holderPtr)
}
