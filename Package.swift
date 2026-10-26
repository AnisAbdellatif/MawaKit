// swift-tools-version: 6.0
// Swift package for MawaKit. Run `make swift` first: it builds the Rust core into
// bindings/swift/MawaKitFFI.xcframework and generates the Swift API.
import PackageDescription

let package = Package(
    name: "MawaKit",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "MawaKit", targets: ["MawaKit"]),
    ],
    targets: [
        .binaryTarget(name: "MawaKitFFI", path: "bindings/swift/MawaKitFFI.xcframework"),
        .target(
            name: "MawaKit",
            dependencies: ["MawaKitFFI"],
            path: "bindings/swift/Sources/MawaKit"
        ),
        .testTarget(
            name: "MawaKitTests",
            dependencies: ["MawaKit"],
            path: "bindings/swift/Tests/MawaKitTests"
        ),
    ]
)
