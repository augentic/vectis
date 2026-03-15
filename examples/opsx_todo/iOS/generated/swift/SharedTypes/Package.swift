// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "SharedTypes",
    products: [
        .library(
            name: "SharedTypes",
            targets: ["SharedTypes"]
        )
    ],
    targets: [
        .target(
            name: "Serde",
            dependencies: []
        ),
        .target(
            name: "SharedTypes",
            dependencies: ["Serde"]
        ),
    ]
)
