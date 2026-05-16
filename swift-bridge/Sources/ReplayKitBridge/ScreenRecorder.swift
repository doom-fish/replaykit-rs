import Foundation
import ReplayKit

// MARK: - Recorder JSON payloads

struct RKRecorderStatePayload: Encodable {
    let isAvailable: Bool
    let isRecording: Bool
    let isMicrophoneEnabled: Bool
    let isCameraEnabled: Bool
}

struct RKRecordingErrorPayload: Encodable {
    let domain: String
    let code: Int
    let localizedDescription: String
}

// MARK: - Delegate holder

/// Inner class that adopts `RPScreenRecorderDelegate` and forwards events
/// to a Rust refcon callback.
///
/// Layout of the C callback:
///   `callback(refcon, event_json_ptr)` where event_json is a JSON string
///   describing the event.  The callee must NOT free the JSON pointer;
///   it is valid only for the duration of the callback.
final class RKDelegateHolder: NSObject, RPScreenRecorderDelegate {
    typealias Callback = @convention(c) (
        UnsafeMutableRawPointer?,   // refcon
        UnsafePointer<CChar>?       // event JSON (borrow — caller frees nothing)
    ) -> Void

    let callback: Callback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping Callback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    // MARK: RPScreenRecorderDelegate

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
                      "{\"kind\":\"didStopRecording\"}"
        } else {
            payload = "{\"kind\":\"didStopRecording\",\"error\":null}"
        }
        payload.withCString { callback(refcon, $0) }
    }

    func screenRecorderDidChangeAvailability(_ screenRecorder: RPScreenRecorder) {
        let payload = "{\"kind\":\"availabilityChanged\",\"isAvailable\":\(screenRecorder.isAvailable)}"
        payload.withCString { callback(refcon, $0) }
    }
}

// MARK: - Shared recorder handle

/// Returns a *retained* opaque pointer to `RPScreenRecorder.shared()`.
/// The caller must call `rk_screen_recorder_release` when done.
@_cdecl("rk_screen_recorder_shared")
public func rk_screen_recorder_shared() -> UnsafeMutableRawPointer {
    rk_retain(RPScreenRecorder.shared())
}

@_cdecl("rk_screen_recorder_release")
public func rk_screen_recorder_release(_ ptr: UnsafeMutableRawPointer) {
    rk_release(ptr)
}

// MARK: - State getters

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

@_cdecl("rk_screen_recorder_is_camera_enabled")
public func rk_screen_recorder_is_camera_enabled(_ ptr: UnsafeMutableRawPointer) -> Bool {
    rk_borrow(ptr, as: RPScreenRecorder.self).isCameraEnabled
}

@_cdecl("rk_screen_recorder_state_json")
public func rk_screen_recorder_state_json(
    _ ptr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let r = rk_borrow(ptr, as: RPScreenRecorder.self)
    let payload = RKRecorderStatePayload(
        isAvailable: r.isAvailable,
        isRecording: r.isRecording,
        isMicrophoneEnabled: r.isMicrophoneEnabled,
        isCameraEnabled: r.isCameraEnabled
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

@_cdecl("rk_screen_recorder_stop_recording")
public func rk_screen_recorder_stop_recording(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.stopRecording { _, error in
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

/// Installs a delegate on the recorder; returns a *retained* opaque pointer
/// to the `RKDelegateHolder`.  Call `rk_screen_recorder_clear_delegate` to
/// remove and release it.
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

/// Removes the delegate from the recorder and releases the holder.
@_cdecl("rk_screen_recorder_clear_delegate")
public func rk_screen_recorder_clear_delegate(
    _ recorderPtr: UnsafeMutableRawPointer,
    _ holderPtr: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(recorderPtr, as: RPScreenRecorder.self)
    recorder.delegate = nil
    rk_release(holderPtr)
}
