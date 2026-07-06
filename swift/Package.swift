// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "PromptOSApp",
    platforms: [
        .macOS(.v13)
    ],
    targets: [
        .executableTarget(
            name: "PromptOSApp",
            path: "Sources/PromptOSApp",
            exclude: ["Info.plist"]
        ),
    ]
)
