## Why

The `opsx_todo` Crux core is complete with full business logic, offline sync, and SSE support, but it has no platform shell yet. An iOS shell is needed so the app can be built, run, and tested on iPhone and Simulator, validating the core's view model, event dispatch, and all five effect handlers (Render, HTTP, KeyValue, Time, ServerSentEvents) end-to-end.

## What Changes

- Generate a complete SwiftUI iOS shell for the existing `examples/opsx_todo` Crux app.
- The shell renders three ViewModel states: Loading, TodoList, and Error.
- All user interactions (add, edit, toggle, delete, clear completed, filter, navigate) dispatch shell-facing Event variants to the core.
- Effect handlers in `Core.swift` cover Render, HTTP, KeyValue, Time, and ServerSentEvents.
- Build pipeline via Makefile (typegen, package, xcodegen) produces SharedTypes and Shared Swift packages.
- VectisDesign tokens used for all styling; Inject wired for hot reloading.

## Capabilities

### New Capabilities
- `ios-shell`: SwiftUI iOS shell for the opsx_todo Crux app -- Core.swift bridge, ContentView, screen views (Loading, TodoList, Error), app entry point, project.yml, and Makefile.

### Modified Capabilities

(none)

## Impact

- **New files**: `examples/opsx_todo/iOS/` directory with all Swift sources, `project.yml`, and `Makefile`.
- **Dependencies**: VectisDesign Swift package (design-system/ios/), Inject SPM package, SharedTypes and Shared generated packages.
- **No core changes**: The existing `shared/` crate is consumed as-is; no modifications to Rust code.
