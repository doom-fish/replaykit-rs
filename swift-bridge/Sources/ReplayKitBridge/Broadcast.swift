import Foundation
import ReplayKit
import AppKit

// MARK: - BroadcastActivityController (macOS 11+)

/// Returns a retained pointer to an `RPBroadcastActivityController`.
/// The caller supplies the origin point and optionally a window pointer
/// and preferred extension bundle ID.
///
/// On completion, the Swift side calls `completionCallback(refcon, controllerPtr, errorJson)`.
/// `controllerPtr` is a *retained* pointer to an `RPBroadcastController` (must be released),
/// or NULL on error.  `errorJson` is NULL on success.
@_cdecl("rk_broadcast_activity_controller_show")
public func rk_broadcast_activity_controller_show(
    _ originX: Double,
    _ originY: Double,
    _ windowPtr: UnsafeMutableRawPointer?,
    _ preferredExtensionJson: UnsafePointer<CChar>?,
    _ refcon: UnsafeMutableRawPointer?,
    _ completionCallback: @convention(c) (
        UnsafeMutableRawPointer?,   // refcon
        UnsafeMutableRawPointer?,   // RPBroadcastController* (retained) or NULL
        UnsafeMutablePointer<CChar>? // error JSON or NULL
    ) -> Void
) {
    let point = CGPoint(x: originX, y: originY)
    let window: NSWindow? = windowPtr.map { rk_borrow($0, as: NSWindow.self) }
    let preferredExt: String? = preferredExtensionJson.map { String(cString: $0) }

    RPBroadcastActivityController.showBroadcastPicker(
        at: point,
        from: window,
        preferredExtensionIdentifier: preferredExt
    ) { controller, error in
        if let error {
            let ns = error as NSError
            let payload = RKFrameworkErrorPayload(
                domain: ns.domain,
                code: ns.code,
                localizedDescription: ns.localizedDescription
            )
            let json = (try? rkEncodeJSON(payload)) ?? "{}"
            json.withCString { completionCallback(refcon, nil, UnsafeMutablePointer(mutating: $0)) }
        } else if let controller {
            completionCallback(refcon, rk_retain(controller), nil)
        } else {
            let msg = "{\"kind\":\"framework\",\"domain\":\"RPRecordingErrorDomain\",\"code\":-1,\"localizedDescription\":\"no controller returned\"}"
            msg.withCString { completionCallback(refcon, nil, UnsafeMutablePointer(mutating: $0)) }
        }
    }
}

// MARK: - BroadcastController

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
