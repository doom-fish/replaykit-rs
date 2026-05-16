import CoreMedia
import Foundation
import ReplayKit

private let RKSampleBufferEvent: Int32 = 1
private let RKSampleBufferErrorEvent: Int32 = 2

struct RKSampleBufferPayload: Encodable {
    let bufferType: Int
    let numSamples: Int
    let dataIsReady: Bool
    let presentationTimeSeconds: Double?
    let durationSeconds: Double?
    let videoOrientation: UInt32?
}

private func rkTimeSeconds(_ time: CMTime) -> Double? {
    guard time.isValid, !time.isIndefinite else {
        return nil
    }
    let seconds = CMTimeGetSeconds(time)
    return seconds.isFinite ? seconds : nil
}

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
    if #available(macOS 11.0, *) {
        return true
    }
    return false
}

@_cdecl("rk_screen_recorder_start_capture")
public func rk_screen_recorder_start_capture(
    _ ptr: UnsafeMutableRawPointer,
    _ callback: @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafeMutablePointer<CChar>?
    ) -> Void,
    _ refcon: UnsafeMutableRawPointer?,
    _ outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    return rkBlockOnAsync(
        work: {
            try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                recorder.startCapture { sampleBuffer, bufferType, error in
                    if let error {
                        callback(refcon, RKSampleBufferErrorEvent, rkOwnedErrorCString(error))
                        return
                    }
                    let payload = RKSampleBufferPayload(
                        bufferType: bufferType.rawValue,
                        numSamples: CMSampleBufferGetNumSamples(sampleBuffer),
                        dataIsReady: CMSampleBufferDataIsReady(sampleBuffer),
                        presentationTimeSeconds: rkTimeSeconds(CMSampleBufferGetPresentationTimeStamp(sampleBuffer)),
                        durationSeconds: rkTimeSeconds(CMSampleBufferGetDuration(sampleBuffer)),
                        videoOrientation: rkSampleBufferOrientation(sampleBuffer)
                    )
                    let json = (try? rkEncodeJSON(payload)) ?? "{}"
                    callback(refcon, RKSampleBufferEvent, rkCString(json))
                } completionHandler: { error in
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
