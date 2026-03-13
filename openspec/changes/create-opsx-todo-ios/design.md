## Context

The `examples/opsx_todo` app has a complete Crux shared core in Rust, including
three ViewModel variants (`Loading`, `TodoList`, `Error`), four Effect variants
(`Render`, `Http`, `KeyValue`, `ServerSentEvents`), and twelve shell-facing
Event variants. The core is functional and passes all tests. What is missing is
an iOS shell that renders the UI, dispatches user interactions, and performs
platform I/O on behalf of the core.

The project already has:

- An `ios-writer` skill that generates SwiftUI shells from Crux core sources.
- An `ios-reviewer` skill that reviews generated shells for structural and
  quality issues.
- A `VectisDesign` Swift Package at `design-system/ios/` with semantic color,
  typography, spacing, and corner-radius tokens.
- Reference examples documenting the expected project structure, Core.swift
  pattern, and SwiftUI view patterns.

## Goals / Non-Goals

**Goals:**

- Generate a complete, buildable SwiftUI iOS shell for `examples/opsx_todo`.
- Map all ViewModel variants to screen views.
- Handle all Effect variants in `Core.swift`.
- Dispatch all shell-facing Event variants from appropriate views.
- Use `VectisDesign` tokens for all styling (zero hardcoded colors, fonts, or
  spacing).
- Include hot reloading support via the Inject library.
- Pass ios-reviewer checks with no Critical findings.

**Non-Goals:**

- Widget, watch, or macOS target support.
- Push notifications or background sync.
- Localization / internationalization (strings are hardcoded in English).
- Custom animations or transitions beyond default SwiftUI.
- Changes to the existing shared crate or design system tokens.

## Decisions

### 1. Use the ios-writer skill in Create Mode

**Decision**: Invoke the `ios-writer` skill's Create Mode process rather than
hand-writing the shell.

**Rationale**: The skill encodes the canonical Crux iOS shell pattern, including
effect handler templates, view patterns, and build configuration. Using it
ensures consistency with other shells in the project and reduces the chance of
structural errors that the ios-reviewer would flag.

**Alternative considered**: Writing the shell manually would give more control
over layout but risks deviating from the established patterns that the reviewer
checks against.

### 2. Single-page navigation (no NavigationStack)

**Decision**: Use a flat `switch core.view` in ContentView without
NavigationStack or tabs, since the app has only one Route (`TodoList`).

**Rationale**: The app has three ViewModel states (`Loading`, `TodoList`,
`Error`) but only one shell-navigable Route. Navigation chrome adds complexity
without value for a single-destination app. If the core adds Routes later, the
shell can be updated to add NavigationStack.

**Alternative considered**: Wrapping in NavigationStack now would be
forward-compatible but adds unnecessary structure for a single-route app.

### 3. SSE effect handler using URLSession async bytes

**Decision**: Implement the `ServerSentEvents` effect handler using
`URLSession.bytes(from:)` and a long-running `Task` that parses SSE frames.

**Rationale**: The core emits `SseRequest::Connect(url)` and
`SseRequest::Disconnect` operations. URLSession's async bytes API provides a
clean streaming interface without third-party dependencies. The handler creates
a `Task` on connect and cancels it on disconnect.

**Alternative considered**: Using a third-party EventSource library would
handle reconnection and parsing, but adds a dependency for minimal benefit
given the core already manages reconnection logic.

### 4. KeyValue effect handler using UserDefaults

**Decision**: Implement the `KeyValue` effect handler via `UserDefaults` with
JSON data encoding.

**Rationale**: The todo app persists a small blob (item list + pending ops).
UserDefaults is the simplest storage mechanism on iOS for small key-value data.
The core serializes state to `Vec<u8>` / JSON before sending the KV operation,
so the shell stores raw `Data`.

**Alternative considered**: File-based storage or SQLite would support larger
data but adds complexity. UserDefaults is sufficient for a todo list.

## Risks / Trade-offs

- **[Risk] SSE parsing edge cases** → The custom SSE parser in the shell may
  not handle all frame formats. Mitigation: the core already handles malformed
  SSE data gracefully by ignoring unparseable events.

- **[Risk] cargo-xcode version mismatch** → The shared.xcodeproj must be
  regenerated with `cargo xcode`. Mitigation: the Makefile includes a `setup`
  target that runs the necessary generation steps.

- **[Trade-off] UserDefaults vs file storage** → UserDefaults has a practical
  size limit (~1MB on iOS). Acceptable for a todo app but would need revisiting
  for a real production app with large data sets.
