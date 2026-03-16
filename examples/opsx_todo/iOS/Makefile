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

# Step 2: Build XCFramework via cargo-swift.
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
