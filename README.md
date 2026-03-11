# Vectis

Toolkit for building applications with a user interface.

## Project Goals

- Support as many runtime platforms as possible, focusing on Web Browser, iOS and Android devices without excluding Windows, MacOS or Linux desktops.
- Contain all of the application behaviour in a shared core that can be tested independently of the runtime platform.
- Has a very opinionated application structure that makes it easier for AI code generation to get right.

## CRUX

The project goals are also shared by the [CRUX](https://github.com/redbadger/crux) framework and is written in Rust, the portable, fast and safe programming language favoured by our Augentic frameworks. So this toolkit targets CRUX code generation for the core of applications.

Familiarize yourself with how CRUX works by scanning the [documentation](https://docs.rs/crux_core/latest/crux_core/).

## Creating a Crux App

App generation uses a spec-driven workflow powered by
[OpenSpec](https://github.com/Fission-AI/OpenSpec). Each app is an OpenSpec
**change** that uses the shared `crux-app` schema to produce a proposal,
app-spec, and tasks. The apply phase invokes the `core-writer` skill to
generate a buildable `shared` crate with business logic, state management,
side-effect orchestration, and tests. No shell code (iOS, Android, Web) is
generated; separate skills handle those.

**Prerequisites:** Install the OpenSpec CLI.

```bash
npm install -g @fission-ai/openspec@latest
```

#### Create a new app

1. **Create the change.** Give it a kebab-case name describing what you are building:

   ```bash
   openspec new change create-my-app --schema crux-app
   ```

   This scaffolds `openspec/changes/create-my-app/` with a `.openspec.yaml` that
   binds it to the `crux-app` schema.

2. **Generate the artifacts.** Ask the agent to propose the change:

   > /opsx:propose create-my-app

   Or describe what you want and let the agent fill in the artifacts:

   > Propose a Crux app called "Weather" that fetches forecasts from a REST API
   > and displays them. Put it in `examples/weather`.

   The agent produces three artifacts in dependency order:

   | Artifact | Purpose |
   |---|---|
   | `proposal.md` | App concept, motivation, target directory, capabilities overview |
   | `app-spec.md` | Full app specification in core-writer format (the contract) |
   | `tasks.md` | Implementation checklist -- create directory, invoke core-writer, verify build |

3. **Review.** Read through the artifacts in `openspec/changes/create-my-app/`.
   Edit them by hand or ask the agent to revise before proceeding.

4. **Apply.** Generate the code:

   > /opsx:apply

   The agent works through the tasks: copies the spec into the target directory,
   invokes the core-writer skill in Create Mode, verifies with `cargo check`,
   `cargo test`, and `cargo clippy`, and then runs the core-reviewer skill.

   The code review runs three passes:
   - **Structural** -- missing `render()` calls, serde derives, input validation
   - **Logic** -- state machine completeness, operation coalescing, race conditions,
     conflict-resolution gaps, spec gap detection
   - **Quality** -- `unwrap()`/`expect()` in production, error handling, function length

   Critical and Warning findings are addressed before proceeding.

5. **Archive** (optional). Once you are satisfied with the output:

   > /opsx:archive

#### Update an existing app

To modify an app that was previously generated:

1. Create a new change describing the update:

   ```bash
   openspec new change update-my-app --schema crux-app
   ```

2. In the proposal, reference the existing app and describe what is changing.

3. In the app-spec, provide the **full desired state** of the application (not
   a diff). The core-writer skill compares the spec against the existing code
   and makes targeted edits in Update Mode.

4. Apply and verify as above.

#### Creating multiple apps

The `crux-app` schema is the shared orchestration. Each app is simply a
different change, all following the same workflow:

```bash
openspec new change create-todo-app     --schema crux-app
openspec new change create-weather-app  --schema crux-app
openspec new change create-notes-app    --schema crux-app
```

The default `spec-driven` schema remains available for non-Crux changes to the
project (e.g. documentation, tooling, infrastructure).

#### Check status

```bash
# List all active changes
openspec list

# See artifact completion for a specific change
openspec status --change create-my-app
```

## Spec Format

The app-spec artifact follows a markdown format. A template is at
`.cursor/skills/core-writer/app-spec-template.md`. Required sections:

| Section | What to include |
|---|---|
| **Overview** | App name and a one-line summary of its purpose. |
| **Features** | Every user action and its expected outcome. |
| **Data Model** | The internal state the app tracks. |
| **User Interface** | What the user sees on each view -- focus on data, not styling. |
| **Views** | Every distinct screen/page. Note which are shell-navigable vs internal. |
| **Capabilities** | Which external capabilities the app needs (see table below). |
| **API Details** | HTTP endpoints, methods, request/response shapes. Omit if no HTTP. |
| **Business Rules** | Validation rules, constraints, edge-case behaviour. Omit if none. |

### Capabilities

The skill detects which Crux capabilities your app needs from the
**Capabilities** section of your spec:

| Capability | When to include |
|---|---|
| **Render** | Always included automatically |
| **HTTP** (`crux_http`) | App calls a REST API or any remote endpoint |
| **Key-Value** (`crux_kv`) | App persists data locally (offline storage, caching) |
| **Time** (`crux_time`) | App uses timers, delays, intervals, or scheduling |
| **Platform** (`crux_platform`) | App needs to detect the runtime platform or OS |
| **SSE / Streaming** (custom) | App subscribes to server-sent events or live data streams |

## What Gets Generated

| Artifact | Description |
|---|---|
| `Cargo.toml` (workspace root) | Workspace manifest with pinned Crux git dependencies |
| `clippy.toml` | Clippy configuration for allowed duplicate crates |
| `rust-toolchain.toml` | Rust toolchain targeting iOS, Android, macOS, and WASM |
| `spec.md` | Copy of the app specification used to generate (or update) the core |
| `shared/Cargo.toml` | Crate manifest with detected capabilities and feature gates |
| `shared/src/app.rs` | App trait implementation: Model, Event, ViewModel, Effect, `update()`, `view()`, and tests |
| `shared/src/ffi.rs` | FFI scaffolding for UniFFI and wasm-bindgen |
| `shared/src/lib.rs` | Module wiring and re-exports |

Custom capability modules (e.g. `shared/src/sse.rs` for Server-Sent Events)
are generated when needed.

## Reviewing Generated Code

The `core-reviewer` skill at `.cursor/skills/core-reviewer/SKILL.md`
systematically reviews Crux core (Rust `shared` crate) code for issues that
compilers and linters miss. It runs automatically as part of the apply phase
but can also be invoked standalone:

> Use the core-reviewer skill to review `examples/my-app`

> Review `examples/my-app` against `examples/todo` as a reference

The skill applies 30 checks across three categories (structural, logic, and
quality) and produces a severity-graded report. See the skill's `references/`
directory for the full checklist.

## Creating an iOS Shell

iOS shell generation uses the `ios-shell` OpenSpec schema. Each shell is a
change that produces a SwiftUI application wired to the Crux core via UniFFI.

1. **Create the change:**

   ```bash
   openspec new change create-todo-ios --schema ios-shell
   ```

2. **Propose and generate artifacts:**

   > /opsx:propose create-todo-ios

   Or describe what you want:

   > Propose an iOS shell for the todo app at `examples/todo`. Put it in
   > `examples/todo/iOS`.

   The agent produces three artifacts:

   | Artifact | Purpose |
   |---|---|
   | `proposal.md` | Which app, target directory, design system notes |
   | `shell-spec.md` | iOS-specific UI details (navigation, screen customizations) |
   | `tasks.md` | Implementation checklist -- invoke ios-writer, verify build, review |

3. **Apply:**

   > /opsx:apply

   The agent invokes the `ios-writer` skill, which reads the Crux core's
   `app.rs` to extract ViewModel, Event, Effect, and Route types. It then
   generates:

   - `project.yml` -- XcodeGen project configuration
   - `Makefile` -- build automation
   - `Core.swift` -- bridge between SwiftUI and the Rust core
   - SwiftUI screen views -- one per ViewModel variant
   - App entry point with navigation

   All views use the shared `VectisDesign` package for colors, typography,
   and spacing tokens.

   After generation, the `ios-reviewer` skill reviews the shell for:
   - **Structural** -- missing screen views, incomplete effect handlers
   - **Quality** -- concurrency safety, accessibility, design system compliance
   - **Integration** -- Core.swift correctness, build configuration

## Design System

The `design-system/` directory contains a platform-agnostic design token
specification with an iOS Swift Package implementation.

| Path | Purpose |
|---|---|
| `design-system/spec.md` | Color, typography, spacing, and corner radius rules |
| `design-system/tokens.yaml` | Structured design tokens (source of truth) |
| `design-system/ios/` | `VectisDesign` Swift Package used by all iOS shells |

The design system is shared across all apps generated by the ios-writer skill.
Future platform shells (Android, Web) would add their own implementations
under `design-system/` using the same tokens.

## Reviewing Generated iOS Code

The `ios-reviewer` skill at `.cursor/skills/ios-reviewer/SKILL.md`
reviews iOS shell code for structural and quality issues. It runs automatically
as part of the ios-shell apply phase but can also be invoked standalone:

> Use the ios-reviewer skill to review `examples/my-app`

## Examples

The `examples/` directory contains generated apps:

| Directory | Description |
|---|---|
| `examples/todo` | Offline-first to-do list with sync, SSE, and conflict resolution |

## Developer Setup

- [Install Rust](https://rust-lang.org/tools/install/)
- [Install Cursor](https://cursor.com/home)
- Install the [Rust Analyzer](https://open-vsx.org/extension/rust-lang/rust-analyzer) Cursor extension
- Install [OpenSpec](https://github.com/Fission-AI/OpenSpec) for the spec-driven workflow: `npm install -g @fission-ai/openspec@latest`

### iOS/MacOS Development

[Install Xcode command line tools](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)

```shell
# Builder for Swift projects without needing Xcode UI
brew install xcode-build-server

# Pretty print formatter for `xcodebuild` command output in Cursor terminal
brew install xcbeautify

# Allow for advanced formatting and language features
brew install swiftformat

# Generate Xcode projects from declarative YAML (project.yml)
brew install xcodegen

# Generate Xcode sub-project for the Rust shared library
cargo install cargo-xcode --version 1.7.0
```

Install the [Swift Language Support](https://open-vsx.org/extension/chrisatwindsurf/swift-vscode)
Install the [SweetPad](https://marketplace.visualstudio.com/items?itemName=SweetPad.sweetpad) Cursor extension to link Cursor to Xcode.
