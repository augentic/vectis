# Vectis

Toolkit for building applications with a user interface.

## Project Goals

- Support as many runtime platforms as possible, focusing on Web Browser, iOS and Android devices without excluding Windows, MacOS or Linux desktops.
- Contain all of the application behaviour in a shared core that can be tested independently of the runtime platform.
- Has a very opinionated application structure that makes it easier for AI code generation to get right.

## CRUX

The project goals are also shared by the [CRUX](https://github.com/redbadger/crux) framework and is written in Rust, the portable, fast and safe programming language favoured by our Augentic frameworks. So this toolkit targets CRUX code generation for the core of applications.

Familiarize yourself with how CRUX works by scanning the [documentation](https://docs.rs/crux_core/latest/crux_core/).

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

## Creating a Crux App

App generation uses a spec-driven workflow powered by [OpenSpec](https://github.com/Fission-AI/OpenSpec). Each app is an OpenSpec **change** that uses the shared `crux-app` schema to produce a proposal, app-spec, and tasks. The apply phase invokes the `core-writer` skill to generate a buildable `shared` crate with business logic, state management, side-effect orchestration, and tests. No shell code (iOS, Android, Web) is generated; separate skills handle those.

**Prerequisites:** Install the OpenSpec CLI.

```bash
brew install openspec
```

#### Create a new app

1. **Create the change.** Give it a kebab-case name describing what you are building:
  ```bash
   openspec new change create-my-app --schema crux-app
  ```
   This scaffolds `openspec/changes/create-my-app/` with a `.openspec.yaml` that binds it to the `crux-app` schema.
2. **Generate the artifacts.** Ask the agent to propose the change:
  > /opsx:propose create-my-app
  >  Or describe what you want and let the agent fill in the artifacts:
  > Propose a Crux app called "Weather" that fetches forecasts from a REST API and displays them. Put it in `examples/weather`.
  >  The agent produces three artifacts in dependency order:

  | Artifact      | Purpose                                                                        |
  | ------------- | ------------------------------------------------------------------------------ |
  | `proposal.md` | App concept, motivation, target directory, capabilities overview               |
  | `app-spec.md` | Full app specification in core-writer format (the contract)                    |
  | `tasks.md`    | Implementation checklist -- create directory, invoke core-writer, verify build |

3. **Review.** Read through the artifacts in `openspec/changes/create-my-app/`. Edit them by hand or ask the agent to revise before proceeding.
4. **Apply.** Generate the code:
  > /opsx:apply
  >  The agent works through the tasks: copies the spec into the target directory, invokes the core-writer skill in Create Mode, verifies with `cargo check`, `cargo test`, and `cargo clippy`, and then runs the core-reviewer skill. The code review runs three passes:
  - **Structural** -- missing `render()` calls, serde derives, input validation
  - **Logic** -- state machine completeness, operation coalescing, race conditions, conflict-resolution gaps, spec gap detection
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
3. In the app-spec, provide the **full desired state** of the application (not a diff). The core-writer skill compares the spec against the existing code and makes targeted edits in Update Mode.
4. Apply and verify as above.

#### Creating multiple apps

The `crux-app` schema is the shared orchestration. Each app is simply a different change, all following the same workflow:

```bash
openspec new change create-todo-app     --schema crux-app
openspec new change create-weather-app  --schema crux-app
openspec new change create-notes-app    --schema crux-app
```

The default `spec-driven` schema remains available for non-Crux changes to the project (e.g. documentation, tooling, infrastructure).

#### Check status

```bash
# List all active changes
openspec list

# See artifact completion for a specific change
openspec status --change create-my-app
```

## Spec Format

The app-spec artifact follows a markdown format. A template is at `.cursor/skills/core-writer/app-spec-template.md`. Required sections:


| Section            | What to include                                                         |
| ------------------ | ----------------------------------------------------------------------- |
| **Overview**       | App name and a one-line summary of its purpose.                         |
| **Features**       | Every user action and its expected outcome.                             |
| **Data Model**     | The internal state the app tracks.                                      |
| **User Interface** | What the user sees on each view -- focus on data, not styling.          |
| **Views**          | Every distinct screen/page. Note which are shell-navigable vs internal. |
| **Capabilities**   | Which external capabilities the app needs (see table below).            |
| **API Details**    | HTTP endpoints, methods, request/response shapes. Omit if no HTTP.      |
| **Business Rules** | Validation rules, constraints, edge-case behaviour. Omit if none.       |


### Capabilities

The skill detects which Crux capabilities your app needs from the **Capabilities** section of your spec:


| Capability                     | When to include                                           |
| ------------------------------ | --------------------------------------------------------- |
| **Render**                     | Always included automatically                             |
| **HTTP** (`crux_http`)         | App calls a REST API or any remote endpoint               |
| **Key-Value** (`crux_kv`)      | App persists data locally (offline storage, caching)      |
| **Time** (`crux_time`)         | App uses timers, delays, intervals, or scheduling         |
| **Platform** (`crux_platform`) | App needs to detect the runtime platform or OS            |
| **SSE / Streaming** (custom)   | App subscribes to server-sent events or live data streams |


## What Gets Generated


| Artifact                      | Description                                                                                |
| ----------------------------- | ------------------------------------------------------------------------------------------ |
| `Cargo.toml` (workspace root) | Workspace manifest with pinned Crux git dependencies                                       |
| `clippy.toml`                 | Clippy configuration for allowed duplicate crates                                          |
| `rust-toolchain.toml`         | Rust toolchain targeting iOS, Android, macOS, and WASM                                     |
| `spec.md`                     | Copy of the app specification used to generate (or update) the core                        |
| `shared/Cargo.toml`           | Crate manifest with detected capabilities and feature gates                                |
| `shared/src/app.rs`           | App trait implementation: Model, Event, ViewModel, Effect, `update()`, `view()`, and tests |
| `shared/src/ffi.rs`           | FFI scaffolding for UniFFI and wasm-bindgen                                                |
| `shared/src/lib.rs`           | Module wiring and re-exports                                                               |


Custom capability modules (e.g. `shared/src/sse.rs` for Server-Sent Events) are generated when needed.

## Reviewing Generated Code

The `core-reviewer` skill at `.cursor/skills/core-reviewer/SKILL.md` systematically reviews Crux core (Rust `shared` crate) code for issues that compilers and linters miss. **It runs automatically as part of the apply phase** so you don't need to run it directly but it can also be invoked standalone:

> Use the core-reviewer skill to review `examples/my-app`

> Review `examples/my-app` against `examples/todo` as a reference

The skill applies 30 checks across three categories (structural, logic, and quality) and produces a severity-graded report. See the skill's `references/` directory for the full checklist.

For logic issues it can create a new OpenSpec change to be applied.

## Creating an iOS Shell

iOS shell generation uses the `ios-shell` OpenSpec schema. Each shell is a change that produces a SwiftUI application wired to the Crux core via UniFFI.

1. **Create the change.** The convention is `create-{app}-ios` where `{app}` matches the example directory name, using hyphens: For an app in `examples/opsx_todo`, use `create-opsx-todo-ios`.
2. **Propose and generate artifacts:**
  > /opsx:propose create-todo-ios
  >  Or describe what you want:
  > Propose an iOS shell for the todo app at `examples/todo`. Put it in `examples/todo/iOS`.
  >  The agent produces three artifacts:

  | Artifact        | Purpose                                                             |
  | --------------- | ------------------------------------------------------------------- |
  | `proposal.md`   | Which app, target directory, design system notes                    |
  | `shell-spec.md` | iOS-specific UI details (navigation, screen customizations)         |
  | `tasks.md`      | Implementation checklist -- invoke ios-writer, verify build, review |

3. **Apply:**
  > /opsx:apply
  >  The agent invokes the `ios-writer` skill, which reads the Crux core's `app.rs` to extract ViewModel, Event, Effect, and Route types. It then generates:
  - `project.yml` -- XcodeGen project configuration
  - `Makefile` -- build automation
  - `Core.swift` -- bridge between SwiftUI and the Rust core
  - SwiftUI screen views -- one per ViewModel variant
  - App entry point with navigation
   All views use the shared `VectisDesign` package for colors, typography, and spacing tokens.
   After generation, the `ios-reviewer` skill reviews the shell for:
  - **Structural** -- missing screen views, incomplete effect handlers
  - **Quality** -- concurrency safety, accessibility, design system compliance
  - **Integration** -- Core.swift correctness, build configuration
   See [Working with Xcode](#working-with-xcode) for how to open and build the generated shell.

## Design System

The `design-system/` directory contains a platform-agnostic design token specification with an iOS Swift Package implementation.


| Path                        | Purpose                                                               |
| --------------------------- | --------------------------------------------------------------------- |
| `design-system/spec.md`     | Semantic color roles, typography scale, spacing rules, usage guidance |
| `design-system/tokens.yaml` | Concrete token values (single source of truth for code generation)    |
| `design-system/ios/`        | `VectisDesign` Swift Package -- generated from `tokens.yaml`          |


The design system is shared across all apps generated by the ios-writer skill. Future platform shells (Android, Web) would add their own implementations under `design-system/` using the same tokens.

### Updating the Design System

Design system updates follow a three-layer flow:

```
spec.md (describes intent) → tokens.yaml (defines values) → iOS Swift code (generated)
```

**1. Decide what to change.** Read `design-system/spec.md` to understand the current token roles and usage rules. The spec is human-authored and describes the *why* behind each token.

**2. Edit `tokens.yaml`.** This is the single source of truth for all concrete values. Common changes:

- **Change a value** -- edit the token's entry (e.g., change `primary.light` from `"#007AFF"` to `"#0066CC"`).
- **Add a token** -- add a new entry under an existing category. Follow the naming conventions in `spec.md`.
- **Add a category** -- add a new top-level key (e.g., `elevation`) with entries that follow one of the three value shapes: color (`light`/`dark`), font (`size`/`weight`), or scalar (plain number).
- **Remove a token** -- delete the entry. Check downstream shells for references before removing.

**3. Update `spec.md`** if the change is semantic (new roles, changed usage rules, new categories). For pure value tweaks (adjusting a hex color), the spec usually stays the same.

**4. Regenerate the iOS code.** Use the `design-system-writer` skill:

> Use the design-system-writer skill to regenerate the iOS design system

The skill reads `tokens.yaml` and overwrites the Swift files under `design-system/ios/Sources/VectisDesign/`. It then runs `swift build` to verify the package compiles.

The generated Swift files carry a "do not edit manually" comment. All customization goes through `tokens.yaml`.

### Examples

**Change the primary color:**

1. Edit `design-system/tokens.yaml`:
  ```yaml
   colors:
     primary:
       light: "#0066CC"    # was #007AFF
       dark: "#0A84FF"
  ```
2. Ask the agent to regenerate:
  > Use the design-system-writer skill to regenerate the iOS design system

**Add a tertiary color role:**

1. Update `design-system/spec.md` to document the new role and its purpose.
2. Add entries to `design-system/tokens.yaml`:
  ```yaml
   colors:
     # ... existing entries ...
     tertiary:
       light: "#34C759"
       dark: "#30D158"
     tertiaryContainer:
       light: "#D4F5DD"
       dark: "#0A3D1A"
     onTertiary:
       light: "#FFFFFF"
       dark: "#FFFFFF"
     onTertiaryContainer:
       light: "#0A3D1A"
       dark: "#D4F5DD"
  ```
3. Regenerate with the design-system-writer skill.

**Add a new token category (e.g., elevation):**

1. Document the category in `design-system/spec.md`.
2. Add a new top-level key to `design-system/tokens.yaml`:
  ```yaml
   elevation:
     none: 0
     sm: 2
     md: 4
     lg: 8
     xl: 16
  ```
3. Regenerate. The skill detects the scalar value shape, creates `Elevation.swift` with a `VectisElevation` enum, and adds it to `Theme.swift`.

## Reviewing Generated iOS Code

The `ios-reviewer` skill at `.cursor/skills/ios-reviewer/SKILL.md` reviews iOS shell code for structural and quality issues. It runs automatically as part of the ios-shell apply phase but can also be invoked standalone:

> Use the ios-reviewer skill to review `examples/my-app`

## Examples

The `examples/` directory contains generated apps:


| Directory       | Description                                                      |
| --------------- | ---------------------------------------------------------------- |
| `examples/todo` | Offline-first to-do list with sync, SSE, and conflict resolution |


## Working with Xcode

After generating an iOS shell, the `iOS/` directory contains a `project.yml` (XcodeGen spec) and a `Makefile` -- but no `.xcodeproj` yet. The Xcode project file is generated and gitignored; `project.yml` is the source of truth.

**First-time setup:**

```bash
cd examples/my-app/iOS
make setup
```

This runs two steps: `cargo xcode` in the `shared/` crate (producing `shared/shared.xcodeproj` for the Rust library), then `xcodegen` in the `iOS/` directory (producing `{AppName}.xcodeproj` for the Swift app).

**Open the generated `.xcodeproj`:**

```bash
open MyApp.xcodeproj
```

The project name matches the app name declared in `project.yml`. From here you can build, run on a simulator, and use SwiftUI previews.

**Common mistakes to avoid:**

- Do **not** open `shared/shared.xcodeproj` -- that is the Rust library's Xcode sub-project created by `cargo xcode`, not the iOS app.
- Do **not** look for a `.xcworkspace` -- the ios-writer does not generate one. The single `.xcodeproj` references the shared library as a dependency.
- If Xcode gets into a bad state or creates stray scaffolding files, delete the `.xcodeproj` and regenerate it:
  ```bash
  rm -rf MyApp.xcodeproj
  make xcode
  ```
  Because the project file is fully derived from `project.yml`, this is always safe.

**Build from the command line:**

```bash
make build    # builds for iPhone 16 simulator via xcodebuild
```

