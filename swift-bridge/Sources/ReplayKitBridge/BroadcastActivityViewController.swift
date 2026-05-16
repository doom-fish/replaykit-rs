import AppKit
import Foundation
import ObjectiveC
import ReplayKit

private var rkBroadcastActivityDelegateAssociationKey: UInt8 = 0

final class RKBroadcastActivityDelegateHolder: NSObject, RPBroadcastActivityControllerDelegate {
    typealias Callback = @convention(c) (
        UnsafeMutableRawPointer?,
        UnsafeMutableRawPointer?,
        UnsafeMutablePointer<CChar>?
    ) -> Void

    let callback: Callback
    let refcon: UnsafeMutableRawPointer?
    weak var activityController: RPBroadcastActivityController?

    init(callback: @escaping Callback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func broadcastActivityController(
        _ broadcastActivityController: RPBroadcastActivityController,
        didFinishWith broadcastController: RPBroadcastController?,
        error: Error?
    ) {
        if let error {
            callback(refcon, nil, rkOwnedErrorCString(error))
        } else if let broadcastController {
            callback(refcon, rk_retain(broadcastController), nil)
        } else {
            callback(
                refcon,
                nil,
                rkCString(#"{"kind":"framework","domain":"RPRecordingErrorDomain","code":-1,"localizedDescription":"no broadcast controller returned"}"#)
            )
        }
        activityController?.delegate = nil
        if let activityController {
            objc_setAssociatedObject(
                activityController,
                &rkBroadcastActivityDelegateAssociationKey,
                nil,
                .OBJC_ASSOCIATION_ASSIGN
            )
        }
    }
}

@_cdecl("rk_broadcast_activity_controller_show")
public func rk_broadcast_activity_controller_show(
    _ originX: Double,
    _ originY: Double,
    _ windowPtr: UnsafeMutableRawPointer?,
    _ preferredExtension: UnsafePointer<CChar>?,
    _ refcon: UnsafeMutableRawPointer?,
    _ completionCallback: @convention(c) (
        UnsafeMutableRawPointer?,
        UnsafeMutableRawPointer?,
        UnsafeMutablePointer<CChar>?
    ) -> Void
) {
    let point = CGPoint(x: originX, y: originY)
    let window: NSWindow? = windowPtr.map { rk_borrow($0, as: NSWindow.self) }
    let preferredExtensionIdentifier = preferredExtension.map { String(cString: $0) }

    Task { @MainActor in
        RPBroadcastActivityController.showBroadcastPicker(
            at: point,
            from: window,
            preferredExtensionIdentifier: preferredExtensionIdentifier
        ) { activityController, error in
            if let error {
                completionCallback(refcon, nil, rkOwnedErrorCString(error))
                return
            }
            guard let activityController else {
                completionCallback(
                    refcon,
                    nil,
                    rkCString(#"{"kind":"framework","domain":"RPRecordingErrorDomain","code":-1,"localizedDescription":"no activity controller returned"}"#)
                )
                return
            }
            let holder = RKBroadcastActivityDelegateHolder(callback: completionCallback, refcon: refcon)
            holder.activityController = activityController
            activityController.delegate = holder
            objc_setAssociatedObject(
                activityController,
                &rkBroadcastActivityDelegateAssociationKey,
                holder,
                .OBJC_ASSOCIATION_RETAIN_NONATOMIC
            )
        }
    }
}

@_cdecl("rk_broadcast_activity_view_controller_is_supported")
public func rk_broadcast_activity_view_controller_is_supported() -> Bool {
    false
}

@_cdecl("rk_broadcast_activity_view_controller_unavailable_reason")
public func rk_broadcast_activity_view_controller_unavailable_reason() -> UnsafeMutablePointer<CChar>? {
    rkCString("RPBroadcastActivityViewController is unavailable on macOS; use RPBroadcastActivityController instead")
}
