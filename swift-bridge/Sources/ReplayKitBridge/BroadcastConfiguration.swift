import Foundation

@_cdecl("rk_broadcast_configuration_is_supported")
public func rk_broadcast_configuration_is_supported() -> Bool {
    false
}

@_cdecl("rk_broadcast_configuration_unavailable_reason")
public func rk_broadcast_configuration_unavailable_reason() -> UnsafeMutablePointer<CChar>? {
    rkCString("RPBroadcastConfiguration is deprecated and unavailable on macOS")
}
