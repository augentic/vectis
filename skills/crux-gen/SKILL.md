---
name: crux-gen
description: Generate a Rust project based on the Crux framework for user interface applications.
argument-hint: [project-dir?]
allowed-tools: Read, Write, Edit, Bash
model: opus
disable-model-invocation: true
user-invocable: true
context: fork
agent: general-purpose
---

# Crux Core Application Generator

Generate a buildable Crux core (`shared` crate) for a multi-platform application.
The core contains all business logic, state management, and side-effect orchestration.
No shell code (iOS, Android, Web) is generated -- separate skills handle those.

This skill targets the **Crux 0.17+ API** (master branch). Dependencies use git references
until the crate is published to crates.io.

## Arguments

| Argument | Required | Description |
|---|---|---|
| `project-dir` | No | Directory to create the project in. Defaults to current directory. |

The user provides a **natural-language description** of the application they want to build.
The skill derives all types and structure from that description.

## Derived Arguments

The following are inferred from the user's description. Do **not** prompt for them unless
the description is too ambiguous to proceed.

| Derived | How to infer | Example |
|---|---|---|
| **App struct name** | PascalCase noun from the app concept | `TodoApp`, `NoteEditor`, `Counter` |
| **Model** | Internal state fields from described features | `todos: Vec<Todo>`, `filter: Filter` |
| **Event** | User actions + internal callback variants | `AddTodo(String)`, `Fetched(Result<...>)` |
| **ViewModel** | What the UI needs to display | `items: Vec<TodoView>`, `count: String` |
| **Capabilities** | Inferred from feature keywords (see below) | Render + HTTP + KV |

### Capability Detection

Always include **Render**. Add others based on keywords in the description:

| Capability | Trigger keywords |
|---|---|
| **HTTP** (`crux_http`) | API, REST, fetch, remote, server, endpoint, backend |
| **Key-Value** (`crux_kv`) | persist, storage, cache, offline, local state, save, store |
| **SSE / Streaming** (custom) | real-time, live, server-sent events, push, stream, subscribe |
| **Time** (`crux_time`) | timer, delay, schedule, timeout, interval, clock |
| **Platform** (`crux_platform`) | platform detection, OS-specific |

If the user describes effects not covered by published capabilities, generate a
custom capability module following the pattern in `references/crux-custom-capabilities.md`.

## Process

### 1. Analyze the user's description

Read the description and identify:
- The core concept and app name
- State that needs to be tracked (Model)
- Actions the user can take (shell-facing Event variants)
- Side-effects needed (determines capabilities and internal Event variants)
- What the UI needs to show (ViewModel)

If the description is too vague to determine Model and Events, ask **one** clarifying question.
Use `[unknown]` tokens for anything genuinely ambiguous rather than guessing.

### 2. Design the type system

Before writing any code, design these types on paper:

1. **Model** -- all internal state. Use newtypes and enums for domain concepts.
   Fields should be `pub(crate)` unless needed externally.
2. **Event** -- split into shell-facing variants (serializable, sent by UI) and
   internal variants (marked `#[serde(skip)]` `#[facet(skip)]`, used as effect callbacks).
3. **ViewModel** -- derive `Facet, Serialize, Deserialize, Clone, Debug, Default`.
   Contains only data the UI needs. Use `String` for formatted display values.
4. **Effect** -- one variant per capability. Annotate with `#[effect(facet_typegen)]`.
5. **Supporting types** -- domain structs/enums used in Model, Event, or ViewModel.

Consult `references/crux-app-pattern.md` for type conventions.

### 3. Generate workspace files

Create the workspace root with two files:

**`{project-dir}/Cargo.toml`** -- workspace manifest:
```toml
[workspace]
members = ["shared"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
crux_core = { git = "https://github.com/redbadger/crux", branch = "master" }
serde = "1.0"
facet = "=0.31"
```

Add capability crates to `[workspace.dependencies]` based on detected capabilities.
See `references/crux-project-config.md` for the full dependency list.

**`{project-dir}/rust-toolchain.toml`** -- toolchain config:
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "rustc-dev"]
targets = [
    "aarch64-apple-darwin",
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "aarch64-linux-android",
    "wasm32-unknown-unknown",
    "x86_64-apple-ios",
]
profile = "minimal"
```

### 4. Generate `shared/Cargo.toml`

Follow the template in `references/crux-project-config.md`. Key points:
- `crate-type = ["cdylib", "lib", "staticlib"]`
- Feature-gate `uniffi` and `wasm_bindgen` dependencies
- Add a `codegen` feature for type generation
- Include only the capability crates actually needed

### 5. Generate `shared/src/app.rs`

This is the heart of the application. Follow `references/crux-app-pattern.md`:

1. Define supporting types (domain structs/enums)
2. Define `Model` with `#[derive(Default)]`
3. Define `ViewModel` with `#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default)]`
4. Define `Event` with shell and internal variants
5. Define `Effect` enum with `#[effect(facet_typegen)]`
6. If using HTTP, add type alias: `type Http = crux_http::Http<Effect, Event>;`
7. If using KV, add type alias: `type KeyValue = crux_kv::KeyValue<Effect, Event>;`
8. Implement `App` trait with `update()` and `view()`
9. Write `#[cfg(test)] mod tests` with at least one test per Event variant

For `update()` logic, consult `references/crux-command-api.md` for command patterns.
For testing, consult `references/crux-testing-patterns.md`.

### 6. Generate `shared/src/ffi.rs`

Follow `references/crux-ffi-scaffolding.md` exactly. The `CoreFFI` struct is identical
across all apps except for the `Bridge<AppType>` generic parameter.

### 7. Generate custom capability modules (if needed)

If SSE or other custom capabilities are needed, generate them as separate modules.
Follow `references/crux-custom-capabilities.md` for the pattern.

### 8. Generate `shared/src/lib.rs`

Wire everything together:
```rust
mod app;
pub mod ffi;

pub use app::*;
pub use crux_core::Core;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
```

Add `pub mod {capability};` for any custom capability modules.

### 9. Verify

Run `cargo check` in the project directory. If it fails:
1. Read the error output carefully
2. Fix the issue in the relevant file
3. Re-run `cargo check`
4. Repeat until clean

Then run `cargo test` to verify tests pass.

## Reference Documentation

Consult these references during generation. Do not deviate from the patterns they describe.

| Reference | Purpose |
|---|---|
| `references/crux-app-pattern.md` | App trait, Model, Event, ViewModel, Effect type conventions |
| `references/crux-command-api.md` | Command creation, chaining, combining, async context |
| `references/crux-capabilities.md` | HTTP and KV capability APIs |
| `references/crux-custom-capabilities.md` | Building custom Operation + capability (SSE example) |
| `references/crux-testing-patterns.md` | Testing effects, events, resolving requests |
| `references/crux-ffi-scaffolding.md` | CoreFFI struct, uniffi, wasm-bindgen |
| `references/crux-project-config.md` | Cargo workspace, toolchain, features, dependencies |

## Examples

See `references/examples/` for complete worked examples:

| Example | Capabilities | Demonstrates |
|---|---|---|
| `01-simple-counter.md` | Render | Minimal app, state updates, basic testing |
| `02-http-counter.md` | Render + HTTP | API calls, optimistic updates, effect testing |
| `03-kv-notes.md` | Render + KV | Local persistence, serialization, CRUD |

## Error Handling

| Error | Resolution |
|---|---|
| `cargo check` fails with unresolved import | Verify capability crate is in `[workspace.dependencies]` and `shared/Cargo.toml` |
| `Command` type mismatch | Ensure `update()` returns `Command` (no generic params in 0.17+) |
| `facet` derive errors | Ensure `facet = "=0.31"` is pinned exactly; add `#[repr(C)]` to enums |
| `uniffi` build failures | Ensure `uniffi` is behind `feature = "uniffi"` gate, not unconditional |
| Missing `Operation` impl for custom capability | Each custom request type must `impl Operation` with `type Output` |
| `#[serde(skip)]` on Event variant causes deserialization panic | Internal variants must never be sent from the shell; guard with `#[facet(skip)]` too |

## Verification Checklist

Before completing, verify:

- [ ] `cargo check` passes with no errors
- [ ] `cargo test` passes with no failures
- [ ] Every Event variant is handled in `update()`
- [ ] Every `update()` branch returns a `Command` (not `()`)
- [ ] Internal Event variants have `#[serde(skip)]` and `#[facet(skip)]`
- [ ] `ViewModel` derives `Facet, Serialize, Deserialize, Clone, Debug, Default`
- [ ] Effect enum uses `#[effect(facet_typegen)]`
- [ ] `CoreFFI` uses feature-gated `uniffi` and `wasm_bindgen` attributes
- [ ] At least one test per Event variant exists
- [ ] No `unwrap()` in production code paths (allowed in tests)
- [ ] Type aliases defined for each capability: `type Http = crux_http::Http<Effect, Event>;`

## Important Notes

- **0.17 is unreleased**: Use git dependencies. When 0.17 is published to crates.io, update
  the workspace `Cargo.toml` to use versioned dependencies instead.
- **`facet` version pinning**: The `facet` crate must be pinned to `"=0.31"` exactly.
  Other versions may be incompatible with `crux_core`.
- **No `Capabilities` type**: The 0.17 API removes `type Capabilities` and the `caps`
  parameter from `update()`. Do not include them.
- **`Command` has no generic parameters**: Return `Command` not `Command<Effect, Event>`.
- **`#[repr(C)]` on Event enums**: Required by `facet` for enums that cross the FFI boundary.
- **SSE is not a published crate**: It is a custom capability. Generate it inline when needed.
- **Core only**: This skill generates only the `shared` crate. Shell skills are separate.
