// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SysMac",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "SysMac", targets: ["SysMac"])
    ],
    targets: [
        .executableTarget(
            name: "SysMac",
            path: "SysMac",
            exclude: ["SysMac.entitlements"],
            resources: [
                .process("Resources")
            ],
            linkerSettings: [
                .linkedFramework("IOKit"),
                .linkedFramework("Metal"),
            ]
        ),
        .testTarget(
            name: "SysMacTests",
            dependencies: ["SysMac"],
            path: "Tests"
        )
    ]
)
