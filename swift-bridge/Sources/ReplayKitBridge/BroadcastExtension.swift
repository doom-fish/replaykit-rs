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

private func rkRetainExtensionObject<T: AnyObject>(
    _ ptr: UnsafeMutableRawPointer?,
    as type: T.Type,
    _ outObject: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outObject?.pointee = nil
    guard let ptr else {
        return rkReturnBridgeError(outError, .invalidArgument("missing \(type) pointer"))
    }
    guard let object = Unmanaged<AnyObject>.fromOpaque(ptr).takeUnretainedValue() as? T else {
        return rkReturnBridgeError(outError, .invalidArgument("object is not an instance of \(type)"))
    }
    outObject?.pointee = rk_retain(object)
    return RK_OK
}

private func rkBroadcastHandlerUpdateBroadcastURL(
    _ handler: RPBroadcastHandler,
    broadcastURL: UnsafePointer<CChar>?
) throws {
    handler.updateBroadcast(try rkURL(from: broadcastURL, context: "broadcast URL"))
}

@_cdecl("rk_broadcast_extension_context_is_supported")
public func rk_broadcast_extension_context_is_supported() -> Bool {
    true
}

@_cdecl("rk_broadcast_extension_context_retain")
public func rk_broadcast_extension_context_retain(
    _ ptr: UnsafeMutableRawPointer?,
    _ outContext: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    rkRetainExtensionObject(ptr, as: NSExtensionContext.self, outContext, outError)
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
    true
}

@_cdecl("rk_broadcast_handler_retain")
public func rk_broadcast_handler_retain(
    _ ptr: UnsafeMutableRawPointer?,
    _ outHandler: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    rkRetainExtensionObject(ptr, as: RPBroadcastHandler.self, outHandler, outError)
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
    true
}

@_cdecl("rk_broadcast_sample_handler_retain")
public func rk_broadcast_sample_handler_retain(
    _ ptr: UnsafeMutableRawPointer?,
    _ outHandler: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    rkRetainExtensionObject(ptr, as: RPBroadcastSampleHandler.self, outHandler, outError)
}

@_cdecl("rk_broadcast_sample_handler_finish_broadcast_with_error")
public func rk_broadcast_sample_handler_finish_broadcast_with_error(
    _ ptr: UnsafeMutableRawPointer,
    _ domain: UnsafePointer<CChar>,
    _ code: Int64,
    _ localizedDescription: UnsafePointer<CChar>
) {
    let error = NSError(
        domain: String(cString: domain),
        code: Int(clamping: code),
        userInfo: [NSLocalizedDescriptionKey: String(cString: localizedDescription)]
    )
    rk_borrow(ptr, as: RPBroadcastSampleHandler.self).finishBroadcastWithError(error)
}
