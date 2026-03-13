## Why

The `examples/opsx_todo` Crux app has a fully working shared core (Rust) but no
platform shell. Without an iOS shell, the app cannot be run on device or in the
simulator, making it impossible to validate the spec-driven pipeline end-to-end
from core generation through to a working UI.

## What Changes

- Generate a SwiftUI iOS shell for `examples/opsx_todo` at `examples/opsx_todo/iOS/`.
- Wire the shell to the existing shared crate via UniFFI-generated bindings.
- Implement screen views for all three ViewModel variants: `Loading`, `TodoList`, and `Error`.
- Handle all four Effect variants: `Render`, `Http`, `KeyValue`, `ServerSentEvents`.
- Dispatch all shell-facing Event variants from the UI (Initialize, Navigate, SetInput, AddTodo, EditTitle, ToggleCompleted, DeleteTodo, ClearCompleted, SetFilter, RetrySync, ConnectSse, SseDisconnected).
- Use the shared `VectisDesign` package (`design-system/ios/`) for all color, typography, spacing, and corner-radius tokens.
- Set up XcodeGen project configuration and Makefile build automation.

## Capabilities

### New Capabilities

- `ios-shell`: SwiftUI shell implementation for the opsx_todo Crux app, covering navigation, screen views, effect handling, event dispatch, and design system integration.

### Modified Capabilities

_(none)_

## Impact

- **New files**: `examples/opsx_todo/iOS/` directory containing the SwiftUI project, screen views, Core.swift bridge, project.yml (XcodeGen), and Makefile.
- **Dependencies**: Requires `VectisDesign` Swift Package at `design-system/ios/`. Requires `cargo-xcode` for generating `shared.xcodeproj`.
- **Build tooling**: `cargo xcode` must be run in `examples/opsx_todo/` before the iOS project can build, to produce the shared library Xcode project.
- **No changes** to the existing shared crate, design-system tokens, or other examples.
