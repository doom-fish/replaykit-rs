import Foundation

// MARK: - Status codes

let RK_OK: Int32 = 0
let RK_INVALID_ARGUMENT: Int32 = -1
let RK_TIMED_OUT: Int32 = -2
let RK_NOT_SUPPORTED: Int32 = -3
let RK_FRAMEWORK_ERROR: Int32 = -4
let RK_UNKNOWN: Int32 = -99

// MARK: - C-string helpers

@inline(__always)
func rkCString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

@_cdecl("rk_string_free")
public func rk_string_free(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

// MARK: - Raw-pointer retain / release / borrow helpers

@inline(__always)
func rk_retain<T: AnyObject>(_ object: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
func rk_borrow<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as _: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@inline(__always)
func rk_release(_ ptr: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

@_cdecl("rk_object_release")
public func rk_object_release(_ ptr: UnsafeMutableRawPointer) {
    rk_release(ptr)
}

@_cdecl("rk_object_class_name")
public func rk_object_class_name(_ ptr: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let object = Unmanaged<AnyObject>.fromOpaque(ptr).takeUnretainedValue()
    return rkCString(NSStringFromClass(type(of: object)))
}

// MARK: - Error helpers

enum RKBridgeError: Error, CustomStringConvertible {
    case invalidArgument(String)
    case timedOut(String)
    case notSupported(String)
    case unknown(String)

    var statusCode: Int32 {
        switch self {
        case .invalidArgument: return RK_INVALID_ARGUMENT
        case .timedOut: return RK_TIMED_OUT
        case .notSupported: return RK_NOT_SUPPORTED
        case .unknown: return RK_UNKNOWN
        }
    }

    var description: String {
        switch self {
        case .invalidArgument(let msg),
             .timedOut(let msg),
             .notSupported(let msg),
             .unknown(let msg):
            return msg
        }
    }
}

struct RKFrameworkErrorPayload: Encodable {
    let kind = "framework"
    let domain: String
    let code: Int
    let localizedDescription: String
}

func rkStatus(for error: Error) -> Int32 {
    if let bridgeError = error as? RKBridgeError {
        return bridgeError.statusCode
    }
    return RK_FRAMEWORK_ERROR
}

func rkEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let data = try JSONEncoder().encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw RKBridgeError.unknown("failed to encode JSON as UTF-8")
    }
    return string
}

func rkErrorMessage(_ error: Error) -> String {
    if let bridgeError = error as? RKBridgeError {
        return bridgeError.description
    }
    let nsError = error as NSError
    let payload = RKFrameworkErrorPayload(
        domain: nsError.domain,
        code: nsError.code,
        localizedDescription: nsError.localizedDescription
    )
    return (try? rkEncodeJSON(payload)) ?? nsError.localizedDescription
}

func rkOwnedErrorCString(_ error: Error) -> UnsafeMutablePointer<CChar>? {
    rkCString(rkErrorMessage(error))
}

func rkPopulateError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    with error: Error
) {
    outError?.pointee = rkOwnedErrorCString(error)
}

@discardableResult
func rkReturnBridgeError(
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ error: RKBridgeError
) -> Int32 {
    rkPopulateError(outError, with: error)
    return error.statusCode
}

// MARK: - JSON helpers

func rkJSONCompatibleObject(_ value: Any) -> Any {
    switch value {
    case let string as String:
        return string
    case let number as NSNumber:
        return number
    case let url as URL:
        return url.absoluteString
    case let data as Data:
        return data.base64EncodedString()
    case let date as Date:
        return ISO8601DateFormatter().string(from: date)
    case let array as [Any]:
        return array.map(rkJSONCompatibleObject)
    case let set as NSSet:
        return set.allObjects.map(rkJSONCompatibleObject)
    case let dict as [AnyHashable: Any]:
        var mapped: [String: Any] = [:]
        for (key, value) in dict {
            mapped[String(describing: key)] = rkJSONCompatibleObject(value)
        }
        return mapped
    case let dict as NSDictionary:
        var mapped: [String: Any] = [:]
        dict.forEach { key, value in
            mapped[String(describing: key)] = rkJSONCompatibleObject(value)
        }
        return mapped
    case is NSNull:
        return NSNull()
    default:
        return String(describing: value)
    }
}

func rkJSONString(fromJSONObject object: Any) -> String? {
    guard JSONSerialization.isValidJSONObject(object) else {
        return nil
    }
    guard let data = try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys]) else {
        return nil
    }
    return String(data: data, encoding: .utf8)
}

func rkServiceInfoJSON(_ serviceInfo: Any?) -> String? {
    guard let serviceInfo else { return nil }
    return rkJSONString(fromJSONObject: rkJSONCompatibleObject(serviceInfo))
}

func rkFileURL(from pathCString: UnsafePointer<CChar>?) throws -> URL {
    guard let pathCString else {
        throw RKBridgeError.invalidArgument("missing file-system path")
    }
    return URL(fileURLWithPath: String(cString: pathCString))
}

// MARK: - Semaphore / Task helpers

func rkBlockOnMainActorAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping @MainActor () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    onError: @escaping (Error) -> Void
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var result: Result<T, Error>?

    Task { @MainActor in
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        onError(RKBridgeError.timedOut("ReplayKit operation timed out after \(timeoutSeconds) seconds"))
        return RK_TIMED_OUT
    }

    switch result {
    case .success(let value):
        onSuccess(value)
        return RK_OK
    case .failure(let error):
        onError(error)
        return rkStatus(for: error)
    case .none:
        let err = RKBridgeError.unknown("ReplayKit operation completed without a result")
        onError(err)
        return err.statusCode
    }
}

func rkBlockOnAsync<T>(
    timeoutSeconds: Int = 30,
    work: @escaping () async throws -> T,
    onSuccess: @escaping (T) -> Void,
    onError: @escaping (Error) -> Void
) -> Int32 {
    let semaphore = DispatchSemaphore(value: 0)
    var result: Result<T, Error>?

    Task {
        do {
            result = .success(try await work())
        } catch {
            result = .failure(error)
        }
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(timeoutSeconds)) == .success else {
        onError(RKBridgeError.timedOut("ReplayKit operation timed out after \(timeoutSeconds) seconds"))
        return RK_TIMED_OUT
    }

    switch result {
    case .success(let value):
        onSuccess(value)
        return RK_OK
    case .failure(let error):
        onError(error)
        return rkStatus(for: error)
    case .none:
        let err = RKBridgeError.unknown("ReplayKit operation completed without a result")
        onError(err)
        return err.statusCode
    }
}
