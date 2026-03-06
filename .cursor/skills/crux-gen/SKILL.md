---
name: crux-gen
description: Generate a Rust Crux framework project from a spec file. Use when the user wants to scaffold, bootstrap, or generate a Crux application, or mentions crux-gen.
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
| `spec-file` | **Yes** | Path to a markdown file describing the application (see **Spec File** below). |
| `project-dir` | No | Directory to create the project in. Defaults to current directory. |

## Spec File

The user provides a **markdown specification file** that describes the application to build.
A template is available at `app-spec-template.md` in this skill's directory.

The spec file must contain the following sections:

| Section | Purpose | Maps to |
|---|---|---|
| **Overview** | App name and one-line summary | App struct name |
| **Features** | User actions and expected outcomes | Shell-facing Event variants |
| **Data Model** | Internal state to track | Model fields and supporting types |
| **User Interface** | What the UI displays | ViewModel fields |
| **Capabilities** | External I/O the app needs | Effect variants and capability crates |
| **API Details** | HTTP endpoints, request/response shapes (if applicable) | HTTP call sites, response types |
| **Business Rules** | Validation, constraints, edge cases | Validation logic in `update()` |

If a section is missing or too vague, ask **one** clarifying question before proceeding.

## Derived Arguments

The following are inferred from the spec file. Do **not** prompt for them unless
the spec is too ambiguous to proceed.

| Derived | How to infer | Example |
|---|---|---|
| **App struct name** | PascalCase noun from the Overview section | `TodoApp`, `NoteEditor`, `Counter` |
| **Model** | Internal state fields from Data Model section | `todos: Vec<Todo>`, `filter: Filter` |
| **Event** | User actions from Features + internal callback variants from Capabilities | `AddTodo(String)`, `Fetched(Result<...>)` |
| **ViewModel** | Display data from User Interface section | `items: Vec<TodoView>`, `count: String` |
| **Capabilities** | Explicitly listed in Capabilities section (see below) | Render + HTTP + KV |

### Capability Detection

Always include **Render**. Add others based on the Capabilities section of the spec file:

| Capability | Spec file indicators |
|---|---|
| **HTTP** (`crux_http`) | HTTP row marked "Yes", or API Details section present |
| **Key-Value** (`crux_kv`) | Key-Value storage row marked "Yes" |
| **SSE / Streaming** (custom) | Server-Sent Events row marked "Yes" |
| **Time** (`crux_time`) | Timer / Time row marked "Yes" |
| **Platform** (`crux_platform`) | Platform detection row marked "Yes" |

If the spec describes effects not covered by published capabilities, generate a
custom capability module following the pattern in `references/crux-custom-capabilities.md`.

## Process

### 1. Read and analyze the spec file

Read the spec file at the path provided by the user. Extract:
- The core concept and app name (from **Overview**)
- State that needs to be tracked (from **Data Model** -> Model)
- Actions the user can take (from **Features** -> shell-facing Event variants)
- Side-effects needed (from **Capabilities** -> Effect variants and internal Event variants)
- What the UI needs to show (from **User Interface** -> ViewModel)
- API shapes (from **API Details** -> HTTP call sites and response types)
- Validation and constraints (from **Business Rules** -> logic in `update()`)

If a required section is missing or too vague to determine Model and Events, ask **one**
clarifying question. Use `[unknown]` tokens for anything genuinely ambiguous rather than
guessing.

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

### 10. Review for unused dependencies

After the build passes, audit `Cargo.toml` against actual usage:

1. For every non-optional dependency in `[dependencies]`, search `src/` for a corresponding
   `use {crate_name}` or a macro/derive from that crate. Remove any dependency that has no
   matching usage.
2. For every crate in `[dev-dependencies]`, search test modules for usage. Remove any that
   are not referenced.
3. Re-run `cargo check` after removals to confirm nothing was missed.

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
- [ ] No unused dependencies in `Cargo.toml` -- every crate has a matching `use` in `src/`

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
