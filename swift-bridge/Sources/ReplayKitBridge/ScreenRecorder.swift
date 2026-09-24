import AppKit
import Foundation
import ReplayKit

// MARK: - Recorder JSON payloads

struct RKRecorderStatePayload: Encodable {
    let isAvailable: Bool
    let isRecording: Bool
    let isMicrophoneEnabled: Bool
    let isCameraEnabled: Bool
    let cameraPosition: Int
}

struct RKRecordingErrorPayload: Encodable {
    let domain: String
    let code: Int
    let localizedDescription: String
}

private let RKScreenRecorderAvailabilityChangedEvent: Int32 = 1
private let RKScreenRecorderDidStopRecordingEvent: Int32 = 2

// MARK: - Delegate multiplexer

public typealias RKRecorderSummaryCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<CChar>?
) -> Void

public typealias RKRecorderDetailedCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    Bool,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<CChar>?
) -> Void

private enum RKRecorderObserverCallback {
    case summary(RKRecorderSummaryCallback)
    case detailed(RKRecorderDetailedCallback)
}

private func rkRecordingSummaryStopPayload(_ error: Error?) -> String {
    guard let error else {
        return #"{"kind":"didStopRecording","error":null}"#
    }
    let ns = error as NSError
    let inner = RKRecordingErrorPayload(
        domain: ns.domain,
        code: ns.code,
        localizedDescription: ns.localizedDescription
    )
    return (try? rkEncodeJSON(["kind": "didStopRecording",
                               "error": rkEncodeJSON(inner)])) ??
        #"{"kind":"didStopRecording"}"#
}

private final class RKRecorderObserver {
    let callback: RKRecorderObserverCallback
    let context: UnsafeMutableRawPointer?
    let contextRelease: RKContextCallback

    init(
        callback: RKRecorderObserverCallback,
        context: UnsafeMutableRawPointer?,
        contextRetain: RKContextCallback,
        contextRelease: RKContextCallback
    ) {
        self.callback = callback
        self.context = context
        self.contextRelease = contextRelease
        contextRetain(context)
    }

    deinit {
        contextRelease(context)
    }

    func didStopRecording(
        _ screenRecorder: RPScreenRecorder,
        previewViewController: RPPreviewViewController?,
        error: Error?
    ) {
        switch callback {
        case .summary(let summary):
            rkRecordingSummaryStopPayload(error).withCString { summary(context, $0) }
        case .detailed(let detailed):
            detailed(
                context,
                RKScreenRecorderDidStopRecordingEvent,
                screenRecorder.isAvailable,
                previewViewController.map(rk_retain),
                error.flatMap(rkOwnedErrorCString)
            )
        }
    }

    func availabilityChanged(_ screenRecorder: RPScreenRecorder) {
        switch callback {
        case .summary(let summary):
            let payload = #"{"kind":"availabilityChanged","isAvailable":\#(screenRecorder.isAvailable)}"#
            payload.withCString { summary(context, $0) }
        case .detailed(let detailed):
            detailed(
                context,
                RKScreenRecorderAvailabilityChangedEvent,
                screenRecorder.isAvailable,
                nil,
                nil
            )
        }
    }
}

private final class RKScreenRecorderDelegateMux: NSObject, RPScreenRecorderDelegate {
    static let shared = RKScreenRecorderDelegateMux()

    private let lock = NSLock()
    private let slotLock = NSRecursiveLock()
    private var observers: [(token: UInt64, observer: RKRecorderObserver)] = []
    private var lastToken: UInt64 = 0
    private weak var recorder: RPScreenRecorder?

    private func locked<T>(_ body: () -> T) -> T {
        lock.lock()
        defer { lock.unlock() }
        return body()
    }

    func add(_ observer: RKRecorderObserver, to screenRecorder: RPScreenRecorder) -> UInt64 {
        slotLock.lock()
        defer { slotLock.unlock() }
        let token: UInt64 = locked {
            lastToken &+= 1
            observers.append((token: lastToken, observer: observer))
            recorder = screenRecorder
            return lastToken
        }
        if screenRecorder.delegate !== self {
            screenRecorder.delegate = self
        }
        return token
    }

    func remove(_ token: UInt64) {
        slotLock.lock()
        let (removed, isEmpty): (RKRecorderObserver?, Bool) = locked {
            guard let index = observers.firstIndex(where: { $0.token == token }) else {
                return (nil, observers.isEmpty)
            }
            return (observers.remove(at: index).observer, observers.isEmpty)
        }
        if isEmpty, let recorder, recorder.delegate === self {
            recorder.delegate = nil
        }
        slotLock.unlock()
        withExtendedLifetime(removed) {}
    }

    private func snapshot() -> [RKRecorderObserver] {
        locked { observers.map(\.observer) }
    }

    func screenRecorder(
        _ screenRecorder: RPScreenRecorder,
        didStopRecordingWith previewViewController: RPPreviewViewController?,
        error: Error?
    ) {
        for observer in snapshot() {
            observer.didStopRecording(
                screenRecorder,
                previewViewController: previewViewController,
                error: error
            )
        }
    }

    func screenRecorderDidChangeAvailability(_ screenRecorder: RPScreenRecorder) {
        for observer in snapshot() {
            observer.availabilityChanged(screenRecorder)
        }
    }
}

// MARK: - Shared recorder handle

@_cdecl("rk_screen_recorder_shared")
public func rk_screen_recorder_shared() -> UnsafeMutableRawPointer {
    rk_retain(RPScreenRecorder.shared())
}

@_cdecl("rk_screen_recorder_release")
public func rk_screen_recorder_release(_ ptr: UnsafeMutableRawPointer) {
    rk_release(ptr)
}

// MARK: - State getters / setters

@_cdecl("rk_screen_recorder_is_available")
public func rk_screen_recorder_is_available(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPScreenRecorder.self).isAvailable
}

@_cdecl("rk_screen_recorder_is_recording")
public func rk_screen_recorder_is_recording(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPScreenRecorder.self).isRecording
}

@_cdecl("rk_screen_recorder_is_microphone_enabled")
public func rk_screen_recorder_is_microphone_enabled(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPScreenRecorder.self).isMicrophoneEnabled
}

@_cdecl("rk_screen_recorder_set_microphone_enabled")
public func rk_screen_recorder_set_microphone_enabled(_ ptr: UnsafeMutableRawPointer, _ enabled: Bool) {
    rk_borrow(ptr, as: RPScreenRecorder.self).isMicrophoneEnabled = enabled
}

@_cdecl("rk_screen_recorder_is_camera_enabled")
public func rk_screen_recorder_is_camera_enabled(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPScreenRecorder.self).isCameraEnabled
}

@_cdecl("rk_screen_recorder_set_camera_enabled")
public func rk_screen_recorder_set_camera_enabled(_ ptr: UnsafeMutableRawPointer, _ enabled: Bool) {
    rk_borrow(ptr, as: RPScreenRecorder.self).isCameraEnabled = enabled
}

@_cdecl("rk_screen_recorder_camera_position")
public func rk_screen_recorder_camera_position(_ ptr: UnsafeMutableRawPointer) -> Int32 {
    Int32(rk_borrow(ptr, as: RPScreenRecorder.self).cameraPosition.rawValue)
}

@_cdecl("rk_screen_recorder_set_camera_position")
public func rk_screen_recorder_set_camera_position(
    _ ptr: UnsafeMutableRawPointer,
    _ cameraPosition: Int32
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    if let position = RPCameraPosition(rawValue: Int(cameraPosition)) {
        recorder.cameraPosition = position
    }
}

@_cdecl("rk_screen_recorder_camera_preview_view")
public func rk_screen_recorder_camera_preview_view(
    _ ptr: UnsafeMutableRawPointer,
    _ outView: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    outView?.pointee = nil
    guard rkRequireMainThread("RPScreenRecorder.cameraPreviewView", outError) else {
        return RK_MAIN_THREAD_REQUIRED
    }
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    outView?.pointee = recorder.cameraPreviewView.map(rk_retain)
    return RK_OK
}

@_cdecl("rk_ns_view_is_hidden")
public func rk_ns_view_is_hidden(
    _ ptr: UnsafeMutableRawPointer,
    _ outHidden: UnsafeMutablePointer<Bool>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard rkRequireMainThread("NSView.isHidden", outError) else {
        return RK_MAIN_THREAD_REQUIRED
    }
    outHidden?.pointee = rk_borrow(ptr, as: NSView.self).isHidden
    return RK_OK
}

@_cdecl("rk_screen_recorder_state_json")
public func rk_screen_recorder_state_json(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let payload = RKRecorderStatePayload(
        isAvailable: recorder.isAvailable,
        isRecording: recorder.isRecording,
        isMicrophoneEnabled: recorder.isMicrophoneEnabled,
        isCameraEnabled: recorder.isCameraEnabled,
        cameraPosition: recorder.cameraPosition.rawValue
    )
    guard let json = try? rkEncodeJSON(payload) else { return nil }
    return rkCString(json)
}

// MARK: - start / stop recording

@_cdecl("rk_screen_recorder_start_recording")
public func rk_screen_recorder_start_recording(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.startRecording { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume()
                    }
                }
            }
        },
        onLateSuccess: { _ in
            recorder.stopRecording { _, _ in
                recorder.discardRecording {}
            }
        },
        onSuccess: { _ in },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_screen_recorder_stop_recording_with_preview")
public func rk_screen_recorder_stop_recording_with_preview(
    _ ptr: UnsafeMutableRawPointer,
    _ outPreviewController: UnsafeMutablePointer<UnsafeMutableRawPointer?>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let semaphore = DispatchSemaphore(value: 0)
    var previewController: RPPreviewViewController?
    var operationError: Error?

    recorder.stopRecording { preview, error in
        previewController = preview
        operationError = error
        semaphore.signal()
    }

    guard semaphore.wait(timeout: .now() + .seconds(30)) == .success else {
        return rkReturnBridgeError(
            outError,
            .timedOut("ReplayKit operation timed out after 30 seconds")
        )
    }

    if let previewController {
        outPreviewController?.pointee = rk_retain(previewController)
    } else {
        outPreviewController?.pointee = nil
    }

    if let operationError {
        rkPopulateError(outError, with: operationError)
        return rkStatus(for: operationError)
    }

    return RK_OK
}

@_cdecl("rk_screen_recorder_stop_recording")
public func rk_screen_recorder_stop_recording(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    var previewController: UnsafeMutableRawPointer?
    let status = rk_screen_recorder_stop_recording_with_preview(ptr, &previewController, outError)
    if let previewController {
        rk_release(previewController)
    }
    return status
}

@_cdecl("rk_screen_recorder_stop_recording_with_output_url")
public func rk_screen_recorder_stop_recording_with_output_url(
    _ ptr: UnsafeMutableRawPointer,
    _ outputPath: UnsafePointer<CChar>?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let outputURL: URL
    do {
        outputURL = try rkFileURL(from: outputPath)
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }

    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.stopRecording(withOutput: outputURL) { error in
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

@_cdecl("rk_screen_recorder_discard_recording")
public func rk_screen_recorder_discard_recording(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let semaphore = DispatchSemaphore(value: 0)
    recorder.discardRecording {
        semaphore.signal()
    }
    guard semaphore.wait(timeout: .now() + .seconds(30)) == .success else {
        return rkReturnBridgeError(
            outError,
            .timedOut("ReplayKit operation timed out after 30 seconds")
        )
    }
    return RK_OK
}

@_cdecl("rk_screen_recorder_start_clip_buffering")
public func rk_screen_recorder_start_clip_buffering(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.startClipBuffering { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume()
                    }
                }
            }
        },
        onLateSuccess: { _ in recorder.stopClipBuffering { _ in } },
        onSuccess: { _ in },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_screen_recorder_stop_clip_buffering")
public func rk_screen_recorder_stop_clip_buffering(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.stopClipBuffering { error in
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

@_cdecl("rk_screen_recorder_export_clip_to_output_url")
public func rk_screen_recorder_export_clip_to_output_url(
    _ ptr: UnsafeMutableRawPointer,
    _ outputPath: UnsafePointer<CChar>?,
    _ durationSeconds: Double,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let outputURL: URL
    do {
        outputURL = try rkFileURL(from: outputPath)
    } catch {
        rkPopulateError(outError, with: error)
        return rkStatus(for: error)
    }
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.exportClip(to: outputURL, duration: durationSeconds) { error in
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

// MARK: - Delegate registration

@_cdecl("rk_screen_recorder_add_summary_observer")
public func rk_screen_recorder_add_summary_observer(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ callback: RKRecorderSummaryCallback,
    _ context: UnsafeMutableRawPointer?,
    _ contextRetain: RKContextCallback,
    _ contextRelease: RKContextCallback
) -> UInt64 {
    let observer = RKRecorderObserver(
        callback: .summary(callback),
        context: context,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )
    return RKScreenRecorderDelegateMux.shared.add(
        observer,
        to: rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    )
}

@_cdecl("rk_screen_recorder_add_detailed_observer")
public func rk_screen_recorder_add_detailed_observer(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ callback: RKRecorderDetailedCallback,
    _ context: UnsafeMutableRawPointer?,
    _ contextRetain: RKContextCallback,
    _ contextRelease: RKContextCallback
) -> UInt64 {
    let observer = RKRecorderObserver(
        callback: .detailed(callback),
        context: context,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )
    return RKScreenRecorderDelegateMux.shared.add(
        observer,
        to: rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    )
}

@_cdecl("rk_screen_recorder_remove_observer")
public func rk_screen_recorder_remove_observer(_ token: UInt64) {
    RKScreenRecorderDelegateMux.shared.remove(token)
}
