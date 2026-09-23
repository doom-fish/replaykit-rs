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
