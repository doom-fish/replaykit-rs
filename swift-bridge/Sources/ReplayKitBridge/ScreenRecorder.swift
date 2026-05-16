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
    let hasCameraPreviewView: Bool
}

struct RKRecordingErrorPayload: Encodable {
    let domain: String
    let code: Int
    let localizedDescription: String
}

private let RKScreenRecorderAvailabilityChangedEvent: Int32 = 1
private let RKScreenRecorderDidStopRecordingEvent: Int32 = 2

// MARK: - Delegate holders

final class RKDelegateHolder: NSObject, RPScreenRecorderDelegate {
    typealias Callback = @convention(c) (
        UnsafeMutableRawPointer?,
        UnsafePointer<CChar>?
    ) -> Void

    let callback: Callback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping Callback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func screenRecorder(
        _ screenRecorder: RPScreenRecorder,
        didStopRecordingWith previewViewController: RPPreviewViewController?,
        error: Error?
    ) {
        let payload: String
        if let error {
            let ns = error as NSError
            let inner = RKRecordingErrorPayload(
                domain: ns.domain,
                code: ns.code,
                localizedDescription: ns.localizedDescription
            )
            payload = (try? rkEncodeJSON(["kind": "didStopRecording",
                                          "error": rkEncodeJSON(inner)])) ??
                      #"{"kind":"didStopRecording"}"#
        } else {
            payload = #"{"kind":"didStopRecording","error":null}"#
        }
        payload.withCString { callback(refcon, $0) }
    }

    func screenRecorderDidChangeAvailability(_ screenRecorder: RPScreenRecorder) {
        let payload = #"{"kind":"availabilityChanged","isAvailable":\#(screenRecorder.isAvailable)}"#
        payload.withCString { callback(refcon, $0) }
    }
}

final class RKDetailedDelegateHolder: NSObject, RPScreenRecorderDelegate {
    typealias Callback = @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        Bool,
        UnsafeMutableRawPointer?,
        UnsafeMutablePointer<CChar>?
    ) -> Void

    let callback: Callback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping Callback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func screenRecorder(
        _ screenRecorder: RPScreenRecorder,
        didStopRecordingWith previewViewController: RPPreviewViewController?,
        error: Error?
    ) {
        let previewPointer = previewViewController.map(rk_retain)
        callback(
            refcon,
            RKScreenRecorderDidStopRecordingEvent,
            screenRecorder.isAvailable,
            previewPointer,
            error.flatMap(rkOwnedErrorCString)
        )
    }

    func screenRecorderDidChangeAvailability(_ screenRecorder: RPScreenRecorder) {
        callback(
            refcon,
            RKScreenRecorderAvailabilityChangedEvent,
            screenRecorder.isAvailable,
            nil,
            nil
        )
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
public func rk_screen_recorder_camera_preview_view(_ ptr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    guard let view = recorder.cameraPreviewView else { return nil }
    return rk_retain(view)
}

@_cdecl("rk_ns_view_is_hidden")
public func rk_ns_view_is_hidden(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: NSView.self).isHidden
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
        cameraPosition: recorder.cameraPosition.rawValue,
        hasCameraPreviewView: recorder.cameraPreviewView != nil
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
    guard #available(macOS 12.0, *) else {
        return rkReturnBridgeError(
            outError,
            .notSupported("startClipBufferingWithCompletionHandler is unavailable before macOS 12.0")
        )
    }
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
    guard #available(macOS 12.0, *) else {
        return rkReturnBridgeError(
            outError,
            .notSupported("stopClipBufferingWithCompletionHandler is unavailable before macOS 12.0")
        )
    }
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
    guard #available(macOS 12.0, *) else {
        return rkReturnBridgeError(
            outError,
            .notSupported("exportClipToURL is unavailable before macOS 12.0")
        )
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

@_cdecl("rk_screen_recorder_set_delegate")
public func rk_screen_recorder_set_delegate(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> Void,
    _ refcon: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer {
    let recorder = rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    let holder = RKDelegateHolder(callback: callback, refcon: refcon)
    recorder.delegate = holder
    return rk_retain(holder)
}

@_cdecl("rk_screen_recorder_clear_delegate")
public func rk_screen_recorder_clear_delegate(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ holderPtr: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    recorder.delegate = nil
    rk_release(holderPtr)
}

@_cdecl("rk_screen_recorder_set_detailed_delegate")
public func rk_screen_recorder_set_detailed_delegate(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        Bool,
        UnsafeMutableRawPointer?,
        UnsafeMutablePointer<CChar>?
    ) -> Void,
    _ refcon: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer {
    let recorder = rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    let holder = RKDetailedDelegateHolder(callback: callback, refcon: refcon)
    recorder.delegate = holder
    return rk_retain(holder)
}

@_cdecl("rk_screen_recorder_clear_detailed_delegate")
public func rk_screen_recorder_clear_detailed_delegate(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ holderPtr: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    recorder.delegate = nil
    rk_release(holderPtr)
}
