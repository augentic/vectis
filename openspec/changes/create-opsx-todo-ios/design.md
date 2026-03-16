## Context

The `opsx_todo` Crux core at `examples/opsx_todo/shared/` implements a fully offline-capable todo list with REST sync and SSE push updates. The core defines five effect types (Render, Http, KeyValue, Time, ServerSentEvents), three ViewModel variants (Loading, TodoList, Error), and seven shell-facing events. No platform shell exists yet.

The ios-writer skill generates SwiftUI shells for Crux apps. It reads `app.rs` to extract types and produces Core.swift (effect bridge), ContentView (ViewModel switch), screen views, project.yml (XcodeGen), and a Makefile (build pipeline). All shells use the shared VectisDesign package for styling and the Inject library for hot reloading.

## Goals / Non-Goals

**Goals:**
- Generate a buildable iOS shell that renders all three ViewModel states and dispatches all shell-facing events.
- Implement all five effect handlers in Core.swift (Render, HTTP, KeyValue, Time, SSE).
- Use VectisDesign tokens for consistent styling across all views.
- Wire Inject hot reloading in every view for fast development iteration.
- Set up the three-phase Makefile build (typegen → package → xcode).

**Non-Goals:**
- Modifying the existing Crux core -- the shell consumes it as-is.
- Adding platform-specific business logic -- the shell is a thin render/dispatch layer.
- Android or web shells -- this change is iOS only.
- Custom animations or advanced gestures beyond standard SwiftUI patterns.

## Decisions

**1. Use ios-writer skill for generation**
The ios-writer skill handles the full shell generation workflow: reading `app.rs`, producing Swift files, configuring XcodeGen, and setting up the build pipeline. This avoids manual boilerplate and ensures consistency with other Vectis iOS shells.

*Alternative considered*: Manual scaffolding. Rejected because it duplicates the skill's logic and risks structural inconsistencies.

**2. SSE effect handler via AsyncStream**
The ServerSentEvents effect is a custom capability (not from crux_macros). The iOS handler will use an `AsyncStream` wrapping `URLSession` bytes to parse SSE frames and dispatch events back to the core.

*Alternative considered*: Third-party SSE library. Rejected to keep dependencies minimal; URLSession's async bytes API is sufficient for SSE parsing.

**3. CUID generation in the shell**
The `AddItem(id, title)` event expects the shell to supply a client-generated CUID v2. The iOS shell will use a Swift CUID v2 library or a UUID-based fallback.

*Alternative considered*: Ship a Rust CUID crate via UniFFI. Rejected because it adds FFI complexity for a simple ID generator; a Swift-native solution is simpler.

**4. Single-screen navigation**
The core defines only `Route::TodoList`. The shell uses a flat `switch` in ContentView without NavigationStack, since there is only one navigable destination. Navigation infrastructure can be added later if routes expand.

## Risks / Trade-offs

- **SSE parsing complexity** → The custom SSE capability requires manual frame parsing in Swift. Mitigation: follow the established pattern from the core's `sse.rs` module and keep the parser minimal (event-type + data lines only).
- **CUID v2 dependency** → Adding a Swift CUID library introduces a new dependency. Mitigation: if no suitable package exists, fall back to `UUID().uuidString` which the server already accepts (client-supplied IDs).
- **Offline testing gap** → The offline queue and retry timer are core logic, but the shell must correctly persist via KeyValue and re-trigger sync. Mitigation: manual testing on Simulator with network conditioner; the core's unit tests already validate the logic.
