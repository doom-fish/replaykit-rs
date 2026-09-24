import CoreMedia
import Foundation
import ReplayKit

private let RKSampleBufferEvent: Int32 = 1
private let RKSampleBufferErrorEvent: Int32 = 2

public typealias RKCaptureCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    Int32,
    UnsafeMutableRawPointer?,
    Bool,
    UInt32,
    UnsafeMutablePointer<CChar>?
) -> Void

private func rkSampleBufferOrientation(_ sampleBuffer: CMSampleBuffer) -> UInt32? {
    guard let attachment = CMGetAttachment(
        sampleBuffer,
        key: RPVideoSampleOrientationKey as CFString,
        attachmentModeOut: nil
    ) else {
        return nil
    }
    guard let number = attachment as? NSNumber else { return nil }
    return number.uint32Value
}

@_cdecl("rk_sample_buffer_delegate_is_supported")
public func rk_sample_buffer_delegate_is_supported() -> Bool {
    true
}

/// Owns a +1 reference on the Rust `CallbackContext` for as long as the
/// `startCapture` sample-handler closure is alive. ReplayKit retains the
/// sample handler until `stopCapture` completes, so this holder's `deinit` —
/// and the matching `contextRelease` — only runs once no capture callback can
/// still be dispatched on the capture queue.
private final class RKSampleBufferContextHolder {
    let refcon: UnsafeMutableRawPointer?
    let contextRelease: RKContextCallback

    init(
        refcon: UnsafeMutableRawPointer?,
        contextRetain: RKContextCallback,
        contextRelease: @escaping RKContextCallback
    ) {
        self.refcon = refcon
        self.contextRelease = contextRelease
        contextRetain(refcon)
    }

    deinit {
        contextRelease(refcon)
    }
}

@_cdecl("rk_screen_recorder_start_capture")
public func rk_screen_recorder_start_capture(
    _ ptr: UnsafeMutableRawPointer,
    _ callback: RKCaptureCallback,
    _ refcon: UnsafeMutableRawPointer?,
    _ contextRetain: RKContextCallback,
    _ contextRelease: RKContextCallback,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let contextHolder = RKSampleBufferContextHolder(
        refcon: refcon,
        contextRetain: contextRetain,
        contextRelease: contextRelease
    )
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.startCapture { sampleBuffer, bufferType, error in
                    // Capture `contextHolder` strongly so the Rust CallbackContext
                    // outlives every sample callback dispatched on the capture
                    // queue; it is released when ReplayKit frees this closure.
                    withExtendedLifetime(contextHolder) {
                        if let error {
                            callback(refcon, RKSampleBufferErrorEvent, 0, nil, false, 0, rkOwnedErrorCString(error))
                            return
                        }
                        let orientation = rkSampleBufferOrientation(sampleBuffer)
                        callback(
                            refcon,
                            RKSampleBufferEvent,
                            Int32(clamping: bufferType.rawValue),
                            Unmanaged.passRetained(sampleBuffer).toOpaque(),
                            orientation != nil,
                            orientation ?? 0,
                            nil
                        )
                    }
                } completionHandler: { error in
                    if let error {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume()
                    }
                }
            }
        },
        onLateOutcome: { outcome in
            if case .success = outcome {
                recorder.stopCapture { _ in }
            }
        },
        onSuccess: { _ in },
        onError: { rkPopulateError(outError, with: $0) }
    )
}

@_cdecl("rk_screen_recorder_stop_capture")
public func rk_screen_recorder_stop_capture(
    _ ptr: UnsafeMutableRawPointer,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.stopCapture { error in
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
