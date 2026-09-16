// swift-tools-version: 5.9
import PackageDescription

// The id_token hand-off between the host app and its Safari extension
// (add-lingua-connected-clients, design D6). A local package so the logic runs under
// `swift test`, without a simulator or a signed App Group.
let package = Package(
    name: "LinguaSignIn",
    platforms: [.iOS("17.2"), .macOS(.v12)],
    products: [
        .library(name: "LinguaSignIn", targets: ["LinguaSignIn"]),
    ],
    targets: [
        .target(name: "LinguaSignIn"),
        .testTarget(name: "LinguaSignInTests", dependencies: ["LinguaSignIn"]),
    ]
)
