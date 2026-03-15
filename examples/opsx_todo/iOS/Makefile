.PHONY: all setup build clean typegen package xcode

SHARED_DIR := ../shared

all: setup build

setup: typegen package xcode

# Step 1: Generate SharedTypes (domain types with Bincode serialization)
# and Swift UniFFI bindings (CoreFfi class + C header + modulemap).
# Requires: cargo build --features uniffi (library must exist for bindgen).
typegen:
	@cargo build --manifest-path $(SHARED_DIR)/Cargo.toml --features uniffi
	@RUST_LOG=info cargo run --manifest-path $(SHARED_DIR)/Cargo.toml \
		--bin codegen --features codegen,facet_typegen -- \
		--language swift --output-dir generated/swift

# Step 2: Build XCFramework via cargo-swift, then replace its bundled Swift
# bindings with the version-matched ones from Step 1.
# cargo-swift bundles uniffi-bindgen 0.29 which produces wrong symbol names
# when the crate uses uniffi 0.31 proc macros.
package:
	@cd $(SHARED_DIR) && \
		cargo swift package \
			--name Shared \
			--platforms ios \
			--lib-type static \
			--features uniffi && \
		rm -rf ../iOS/generated/swift/Shared && \
		mkdir -p ../iOS/generated/swift/Shared && \
		cp -r Shared/* ../iOS/generated/swift/Shared/ && \
		rm -rf Shared
	@cp generated/swift/shared.swift \
		generated/swift/Shared/Sources/Shared/shared.swift
	@cp generated/swift/sharedFFI.h \
		generated/swift/Shared/RustFramework.xcframework/ios-arm64/headers/RustFramework/sharedFFI.h
	@cp generated/swift/sharedFFI.h \
		generated/swift/Shared/RustFramework.xcframework/ios-arm64_x86_64-simulator/headers/RustFramework/sharedFFI.h
	@cp generated/swift/sharedFFI.modulemap \
		generated/swift/Shared/RustFramework.xcframework/ios-arm64/headers/RustFramework/module.modulemap
	@cp generated/swift/sharedFFI.modulemap \
		generated/swift/Shared/RustFramework.xcframework/ios-arm64_x86_64-simulator/headers/RustFramework/module.modulemap

xcode:
	@xcodegen

build:
	@xcodebuild build \
		-project Todo.xcodeproj \
		-scheme Todo \
		-destination 'generic/platform=iOS Simulator' \
		-configuration Debug \
		CODE_SIGNING_ALLOWED=NO \
		2>&1 | xcbeautify

clean:
	@xcodebuild clean -project Todo.xcodeproj -scheme Todo \
		2>&1 | xcbeautify
	@rm -rf generated
