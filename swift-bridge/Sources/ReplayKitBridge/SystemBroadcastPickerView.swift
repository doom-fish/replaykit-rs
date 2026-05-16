import Foundation

@_cdecl("rk_system_broadcast_picker_view_is_supported")
public func rk_system_broadcast_picker_view_is_supported() -> Bool {
    false
}

@_cdecl("rk_system_broadcast_picker_view_unavailable_reason")
public func rk_system_broadcast_picker_view_unavailable_reason() -> UnsafeMutablePointer<CChar>? {
    rkCString("RPSystemBroadcastPickerView is unavailable on macOS")
}
