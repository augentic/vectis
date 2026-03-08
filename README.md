# Vectis

Toolkit for building applications with a user interface.

## Project Goals

- Support as many runtime platforms as possible, focusing on Web Browser, iOS and Android devices without excluding Windows, MacOS or Linux desktops.
- Contain all of the application behaviour in a shared core that can be tested independently of the runtime platform.
- Has a very opinionated application structure that makes it easier for AI code generation to get right.

## CRUX

The project goals are also shared by the [CRUX](https://github.com/redbadger/crux) framework and is written in Rust, the portable, fast and safe programming language favoured by our Augentic frameworks. So this toolkit targets CRUX code generation for the core of applications.

Familiarize yourself with how CRUX works by scanning the [documentation](https://docs.rs/crux_core/latest/crux_core/)

## Using the core-writer Skill

The `core-writer` skill generates a buildable [Crux](https://github.com/redbadger/crux) `shared` crate from a **markdown specification file** that describes your application. It produces all of the core business logic, state management, and side-effect orchestration -- no shell code (iOS, Android, Web) is generated.

### Quick Start

1. Open this workspace in Cursor.
2. Copy the spec template into your project and fill it out:

   ```bash
   cp skills/core-writer/app-spec-template.md my-todo-app.md
   ```

3. Edit `my-todo-app.md` to describe the app you want to build (see **What You Provide** below).
4. Ask the agent to generate the app, passing the spec file. Reference the skill explicitly so the agent picks it up:

   > Use the core-writer skill with `my-todo-app.md` to generate the app in `my-todo-app/`.

5. The skill will read the spec, derive all types and structure, and generate a complete `shared` crate with tests.
6. Once generation finishes, `cargo check` and `cargo test` are run automatically to verify the output.

### What You Provide

A markdown specification file based on the template at `skills/core-writer/app-spec-template.md`. The template contains the following sections -- fill out each one:

| Section | What to include |
|---|---|
| **Overview** | App name and a one-line summary of its purpose. |
| **Features** | Every user action and its expected outcome (e.g. "Add item -- user enters text and taps Add; a new item appears"). |
| **Data Model** | The internal state the app tracks (e.g. "a list of items, each with id, title, and completed flag"). |
| **User Interface** | What the user sees -- focus on data displayed, not visual styling. |
| **Capabilities** | Which external capabilities the app needs (HTTP, Key-Value storage, Timer, SSE, Platform). |
| **API Details** | If the app uses HTTP: endpoints, methods, request/response shapes. Remove if not applicable. |
| **Business Rules** | Validation rules, constraints, or edge-case behaviour. Remove if none. |

You can optionally specify a project directory (defaults to the current directory).

### What Gets Generated

| Artifact | Description |
|---|---|
| `Cargo.toml` (workspace root) | Workspace manifest with pinned Crux git dependencies |
| `rust-toolchain.toml` | Rust toolchain targeting iOS, Android, macOS, and WASM |
| `shared/Cargo.toml` | Crate manifest with detected capabilities and feature gates |
| `shared/src/app.rs` | App trait implementation: Model, Event, ViewModel, Effect, `update()`, `view()`, and tests |
| `shared/src/ffi.rs` | FFI scaffolding for UniFFI and wasm-bindgen |
| `shared/src/lib.rs` | Module wiring and re-exports |

### Capabilities

The skill detects which Crux capabilities your app needs from the **Capabilities** section of your spec file:

| Capability | When to include |
|---|---|
| **Render** | Always included automatically |
| **HTTP** (`crux_http`) | App calls a REST API or any remote endpoint |
| **Key-Value** (`crux_kv`) | App persists data locally (offline storage, caching) |
| **Time** (`crux_time`) | App uses timers, delays, intervals, or scheduling |
| **Platform** (`crux_platform`) | App needs to detect the runtime platform or OS |
| **SSE / Streaming** (custom) | App subscribes to server-sent events or live data streams |

### Examples

Simple counter (no side effects) -- `counter-spec.md`:

```markdown
# App Specification: Counter

## Overview
A minimal counter app with increment and decrement buttons.

## Features
- **Increment** -- user taps "+"; the count increases by one.
- **Decrement** -- user taps "-"; the count decreases by one.
- **Reset** -- user taps "Reset"; the count returns to zero.

## Data Model
- A single integer count, starting at zero.

## User Interface
- The current count displayed as text (e.g. "Count is: 3").
- Three buttons: "+", "-", and "Reset".

## Capabilities
| Capability | Needed? | Details |
|---|---|---|
| **HTTP** | No | |
| **Key-Value storage** | No | |
| **Timer / Time** | No | |
| **Server-Sent Events** | No | |
| **Platform detection** | No | |
```

> Use core-writer with `counter-spec.md` to generate the app in `my-counter/`.

Notes app with local persistence -- create a spec file describing notes CRUD with Key-Value storage marked "Yes".

Weather dashboard with HTTP -- create a spec file with the API Details section describing the weather endpoint.

## Developer Setup

- [Install Rust](https://rust-lang.org/tools/install/)
- [Install Cursor](https://cursor.com/home)
- Install the [Rust Analyzer](https://open-vsx.org/extension/rust-lang/rust-analyzer) Cursor extension

### iOS/MacOS Development

[Install Xcode command line tools](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)

```shell
# Builder for Swift projects without needing Xcode UI
brew install xcode-build-server

# Pretty print formatter for `xcodebuild` command output in Cursor terminal
brew install xcbeautify

# Allow for advanced formatting and language features
brew install swiftformat
```

Install the [Swift Language Support](https://open-vsx.org/extension/chrisatwindsurf/swift-vscode)
Install the [SweetPad](https://marketplace.visualstudio.com/items?itemName=SweetPad.sweetpad) Cursor extension to link Cursor to Xcode.