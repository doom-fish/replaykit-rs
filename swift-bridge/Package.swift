// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "ReplayKitBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "ReplayKitBridge",
            type: .static,
            targets: ["ReplayKitBridge"])
    ],
    targets: [
        .target(
            name: "ReplayKitBridge",
            path: "Sources/ReplayKitBridge")
    ]
)
