.PHONY: all build clean setup typegen package xcode sim-build

SHARED_DIR := ../shared

all: build

setup: build

# Full build pipeline
build: typegen package xcode

# Phase 1: Generate SharedTypes (domain types as a Swift package)
typegen:
	@echo "Generating SharedTypes..."
	@RUST_LOG=info cargo run --manifest-path $(SHARED_DIR)/Cargo.toml \
		--bin codegen --features codegen,facet_typegen -- \
		--language swift --output-dir generated

# Phase 2: Build Shared (UniFFI bindings as a Swift package via cargo-swift)
package:
	@echo "Building Shared Swift package..."
	@cd $(SHARED_DIR) && \
		cargo swift package --name Shared --platforms ios \
			--lib-type static --features uniffi && \
		rm -rf ../iOS/generated/Shared && \
		mkdir -p ../iOS/generated/Shared && \
		cp -r Shared/* ../iOS/generated/Shared/ && \
		rm -rf Shared

# Phase 3: Generate Xcode project from project.yml
xcode:
	@echo "Generating Xcode project..."
	@xcodegen

# Build the iOS app for Simulator
sim-build:
	@xcodebuild build \
		-project Todo.xcodeproj \
		-scheme Todo \
		-destination 'generic/platform=iOS Simulator' \
		-configuration Debug \
		CODE_SIGNING_ALLOWED=NO \
		2>&1 | xcbeautify

# Clean build artifacts
clean:
	@rm -rf generated/ *.xcodeproj
