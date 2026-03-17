// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "VectisDesign",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "VectisDesign", targets: ["VectisDesign"]),
    ],
    targets: [
        .target(name: "VectisDesign"),
    ]
)
