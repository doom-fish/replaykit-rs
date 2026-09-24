import AppKit
import Foundation
import ReplayKit

// MARK: - Async completions (non-blocking callback-based pattern)

/// Callback for operations that return void
public typealias RKAsyncCompletion = @convention(c) (
    UnsafeRawPointer?,
    Int32,
    UnsafeMutablePointer<CChar>?,
    UnsafeMutableRawPointer
) -> Void

/// Callback for stopRecording that returns a preview controller
public typealias RKAsyncStopRecordingCompletion = RKAsyncCompletion

private func rkCompleteAsync(
    _ callback: RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer,
    _ error: Error?,
    result: () -> UnsafeRawPointer? = { nil }
) {
    if let error {
        callback(nil, rkStatus(for: error), rkOwnedErrorCString(error), ctx)
    } else {
        callback(result(), RK_OK, nil, ctx)
    }
}

private let RKAsyncRecordingTimeoutSeconds = 30

private final class RKBoundedAsyncCall: @unchecked Sendable {
    private let lock = NSLock()
    private var callback: RKAsyncCompletion?
    private let ctx: UnsafeMutableRawPointer

    init(_ callback: @escaping RKAsyncCompletion, _ ctx: UnsafeMutableRawPointer) {
        self.callback = callback
        self.ctx = ctx
        let seconds = RKAsyncRecordingTimeoutSeconds
        DispatchQueue.global().asyncAfter(deadline: .now() + .seconds(seconds)) { [self] in
            complete(RKBridgeError.timedOut("ReplayKit did not finish the operation within \(seconds) seconds"))
        }
    }

    func complete(_ error: Error?, result: () -> UnsafeRawPointer? = { nil }) {
        lock.lock()
        guard let callback else {
            lock.unlock()
            return
        }
        self.callback = nil
        lock.unlock()
        rkCompleteAsync(callback, ctx, error, result: result)
    }
}

// MARK: - startRecording async

@_cdecl("rk_screen_recorder_start_recording_async")
public func rk_screen_recorder_start_recording_async(
    _ ptr: UnsafeMutableRawPointer,
    _ cb: @escaping RKAsyncCompletion,
    _ ctx: UnsafeMutableRawPointer
) {
    let recorder = rk_borrow(ptr, as: RPScreenRecorder.self)
    recorder.startRecording { error in
        rkCompleteAsync(cb, ctx, error)
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
    let call = RKBoundedAsyncCall(cb, ctx)
    recorder.stopRecording { preview, error in
        call.complete(error) {
            preview.map { UnsafeRawPointer(rk_retain($0)) }
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
        rkCompleteAsync(cb, ctx, error)
        return
    }

    let call = RKBoundedAsyncCall(cb, ctx)
    recorder.stopRecording(withOutput: outputURL) { error in
        call.complete(error)
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
    let call = RKBoundedAsyncCall(cb, ctx)
    recorder.discardRecording {
        call.complete(nil)
    }
}
