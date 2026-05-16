import Foundation
import ReplayKit

private struct RKBroadcastingApplicationInfoPayload: Encodable {
    let bundleIdentifier: String
    let displayName: String
    let hasAppIcon: Bool
    let appIconClassName: String?
}

private func rkBroadcastHandlerUpdateServiceInfo(
    _ handler: RPBroadcastHandler,
    serviceInfoJSON: UnsafePointer<CChar>?
) throws {
    let serviceInfo = try rkDictionaryFromJSON(serviceInfoJSON, context: "broadcast service info") ?? [:]
    handler.updateServiceInfo(serviceInfo)
}

private func rkBroadcastHandlerUpdateBroadcastURL(
    _ handler: RPBroadcastHandler,
    broadcastURL: UnsafePointer<CChar>?
) throws {
    handler.updateBroadcast(try rkURL(from: broadcastURL, context: "broadcast URL"))
}

@_cdecl("rk_broadcast_extension_context_is_supported")
public func rk_broadcast_extension_context_is_supported() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_broadcast_extension_context_new")
public func rk_broadcast_extension_context_new() -> UnsafeMutableRawPointer {
    rk_retain(NSExtensionContext())
}

@_cdecl("rk_broadcast_extension_context_load_application_info_json")
public func rk_broadcast_extension_context_load_application_info_json(
    _ ptr: UnsafeMutableRawPointer,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let context = rk_borrow(ptr, as: NSExtensionContext.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<String, Error>) in
                context.loadBroadcastingApplicationInfo { bundleIdentifier, displayName, appIcon in
                    let payload = RKBroadcastingApplicationInfoPayload(
                        bundleIdentifier: bundleIdentifier,
                        displayName: displayName,
                        hasAppIcon: appIcon != nil,
                        appIconClassName: appIcon.map { NSStringFromClass(type(of: $0)) }
                    )
                    do {
                        continuation.resume(returning: try rkEncodeJSON(payload))
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }
        },
        onSuccess: { outJSON?.pointee = rkCString($0) },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_broadcast_extension_context_complete_request_with_broadcast_url")
public func rk_broadcast_extension_context_complete_request_with_broadcast_url(
    _ ptr: UnsafeMutableRawPointer,
    _ broadcastURL: UnsafePointer<CChar>?,
    _ setupInfoJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let context = rk_borrow(ptr, as: NSExtensionContext.self)
        let url = try rkURL(from: broadcastURL, context: "broadcast URL")
        let setupInfo = try rkDictionaryFromJSON(setupInfoJSON, context: "broadcast setup info")
        context.completeRequest(withBroadcast: url, setupInfo: setupInfo)
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_handler_is_supported")
public func rk_broadcast_handler_is_supported() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_broadcast_handler_new")
public func rk_broadcast_handler_new() -> UnsafeMutableRawPointer {
    rk_retain(RPBroadcastHandler())
}

@_cdecl("rk_broadcast_handler_update_service_info")
public func rk_broadcast_handler_update_service_info(
    _ ptr: UnsafeMutableRawPointer,
    _ serviceInfoJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try rkBroadcastHandlerUpdateServiceInfo(
            rk_borrow(ptr, as: RPBroadcastHandler.self),
            serviceInfoJSON: serviceInfoJSON
        )
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_handler_update_broadcast_url")
public func rk_broadcast_handler_update_broadcast_url(
    _ ptr: UnsafeMutableRawPointer,
    _ broadcastURL: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try rkBroadcastHandlerUpdateBroadcastURL(
            rk_borrow(ptr, as: RPBroadcastHandler.self),
            broadcastURL: broadcastURL
        )
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_sample_handler_is_supported")
public func rk_broadcast_sample_handler_is_supported() -> Bool {
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_broadcast_sample_handler_new")
public func rk_broadcast_sample_handler_new() -> UnsafeMutableRawPointer {
    rk_retain(RPBroadcastSampleHandler())
}

@_cdecl("rk_broadcast_sample_handler_update_service_info")
public func rk_broadcast_sample_handler_update_service_info(
    _ ptr: UnsafeMutableRawPointer,
    _ serviceInfoJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try rkBroadcastHandlerUpdateServiceInfo(
            rk_borrow(ptr, as: RPBroadcastSampleHandler.self),
            serviceInfoJSON: serviceInfoJSON
        )
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_sample_handler_update_broadcast_url")
public func rk_broadcast_sample_handler_update_broadcast_url(
    _ ptr: UnsafeMutableRawPointer,
    _ broadcastURL: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        try rkBroadcastHandlerUpdateBroadcastURL(
            rk_borrow(ptr, as: RPBroadcastSampleHandler.self),
            broadcastURL: broadcastURL
        )
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_sample_handler_broadcast_started")
public func rk_broadcast_sample_handler_broadcast_started(
    _ ptr: UnsafeMutableRawPointer,
    _ setupInfoJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let sampleHandler = rk_borrow(ptr, as: RPBroadcastSampleHandler.self)
        let setupInfo = try rkDictionaryFromJSON(setupInfoJSON, context: "broadcast setup info")
        sampleHandler.broadcastStarted(withSetupInfo: setupInfo)
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_sample_handler_broadcast_paused")
public func rk_broadcast_sample_handler_broadcast_paused(_ ptr: UnsafeMutableRawPointer) {
    rk_borrow(ptr, as: RPBroadcastSampleHandler.self).broadcastPaused()
}

@_cdecl("rk_broadcast_sample_handler_broadcast_resumed")
public func rk_broadcast_sample_handler_broadcast_resumed(_ ptr: UnsafeMutableRawPointer) {
    rk_borrow(ptr, as: RPBroadcastSampleHandler.self).broadcastResumed()
}

@_cdecl("rk_broadcast_sample_handler_broadcast_finished")
public func rk_broadcast_sample_handler_broadcast_finished(_ ptr: UnsafeMutableRawPointer) {
    rk_borrow(ptr, as: RPBroadcastSampleHandler.self).broadcastFinished()
}

@_cdecl("rk_broadcast_sample_handler_broadcast_annotated_with_application_info")
public func rk_broadcast_sample_handler_broadcast_annotated_with_application_info(
    _ ptr: UnsafeMutableRawPointer,
    _ applicationInfoJSON: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let sampleHandler = rk_borrow(ptr, as: RPBroadcastSampleHandler.self)
        let applicationInfo = try rkDictionaryFromJSON(
            applicationInfoJSON,
            context: "broadcast application info"
        ) ?? [:]
        sampleHandler.broadcastAnnotated(withApplicationInfo: applicationInfo)
        return RK_OK
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
}

@_cdecl("rk_broadcast_sample_handler_finish_broadcast_with_error")
public func rk_broadcast_sample_handler_finish_broadcast_with_error(
    _ ptr: UnsafeMutableRawPointer,
    _ domain: UnsafePointer<CChar>?,
    _ code: Int64,
    _ localizedDescription: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let sampleHandler = rk_borrow(ptr, as: RPBroadcastSampleHandler.self)
    let error = NSError(
        domain: domain.map { String(cString: $0) } ?? RPRecordingErrorDomain,
        code: Int(clamping: code),
        userInfo: [
            NSLocalizedDescriptionKey: localizedDescription.map { String(cString: $0) } ??
                "Broadcast finished with an error"
        ]
    )
    sampleHandler.finishBroadcastWithError(error)
    outError?.pointee = nil
    return RK_OK
}
