import AppKit
import Foundation
import ReplayKit

// MARK: - Async completions (non-blocking callback-based pattern)

/// Callback for operations that return void
public typealias RKAsyncCompletion = @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void

/// Callback for stopRecording that returns a preview controller
public typealias RKAsyncStopRecordingCompletion = @convention(c) (UnsafeRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer) -> Void

// MARK: - startRecording async

@_cdecl("rk_screen_recorder_start_recording_async")
public func rk_screen_recorder_start_recording_async(
    _ ptr: UnsafeMutableRawPointer,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.startRecording { error in
        if let error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else {
            cb(nil, nil, ctx)
        }
    }
}

// MARK: - stopRecording async

@_cdecl("rk_screen_recorder_stop_recording_async")
public func rk_screen_recorder_stop_recording_async(
    _ ptr: UnsafeMutableRawPointer,
    _ cb: @escaping RKAsyncStopRecordingCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.stopRecording { preview, error in
        if let error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else if let preview {
            cb(rk_retain(preview), nil, ctx)
        } else {
            cb(nil, nil, ctx)
        }
    }
}

// MARK: - stopRecording with output async

@_cdecl("rk_screen_recorder_stop_recording_with_output_async")
public func rk_screen_recorder_stop_recording_with_output_async(
    _ ptr: UnsafeMutableRawPointer,
    _ outputPath: UnsafePointer<CChar>,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    let outputURL: URL
    do {
        outputURL = try rkFileURL(from: outputPath)
    } catch {
        error.localizedDescription.withCString { cb(nil, $0, ctx) }
        return
    }
    
    recorder.stopRecording(withOutput: outputURL) { error in
        if let error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else {
            cb(nil, nil, ctx)
        }
    }
}

// MARK: - startCapture async (returns completion for start, callback for samples)

@_cdecl("rk_screen_recorder_start_capture_async")
public func rk_screen_recorder_start_capture_async(
    _ ptr: UnsafeMutableRawPointer,
    _ sampleCallback: @escaping @convention(c) (
        UnsafeMutableRawPointer?,
        Int32,
        UnsafePointer<CChar>?
    ) -> Void,
    _ sampleCtx: UnsafeMutableRawPointer?,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.startCapture(
        handler: { sampleBuffer, bufferType, error in
            if let error {
                let desc = error.localizedDescription
                desc.withCString { sampleCallback(sampleCtx, 2, $0) }
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
            json.withCString { sampleCallback(sampleCtx, 1, $0) }
        },
        completionHandler: { error in
            if let error {
                error.localizedDescription.withCString { cb(nil, $0, ctx) }
            } else {
                cb(nil, nil, ctx)
            }
        }
    )
}

// MARK: - stopCapture async

@_cdecl("rk_screen_recorder_stop_capture_async")
public func rk_screen_recorder_stop_capture_async(
    _ ptr: UnsafeMutableRawPointer,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.stopCapture { error in
        if let error {
            error.localizedDescription.withCString { cb(nil, $0, ctx) }
        } else {
            cb(nil, nil, ctx)
        }
    }
}

// MARK: - discard recording async

@_cdecl("rk_screen_recorder_discard_recording_async")
public func rk_screen_recorder_discard_recording_async(
    _ ptr: UnsafeMutableRawPointer,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.discardRecording {
        cb(nil, nil, ctx)
    }
}

// MARK: - Helper to get localizedDescription from Error

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
