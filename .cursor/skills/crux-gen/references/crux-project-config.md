# Crux Project Configuration

This reference describes the Cargo workspace layout, dependency management, feature flags,
and toolchain configuration for a Crux core project targeting the 0.17+ API.

## Directory Layout

```
{project-dir}/
    Cargo.toml              # workspace manifest
    rust-toolchain.toml     # toolchain and targets
    shared/
        Cargo.toml          # library manifest
        src/
            lib.rs
            app.rs
            ffi.rs
            {custom}.rs     # optional custom capability modules
```

## Workspace `Cargo.toml`

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

Add capability crates based on what the app needs:

```toml
# Add if using HTTP
crux_http = { git = "https://github.com/redbadger/crux", branch = "master" }

# Add if using Key-Value
crux_kv = { git = "https://github.com/redbadger/crux", branch = "master" }

# Add if using Time
crux_time = { git = "https://github.com/redbadger/crux", branch = "master" }

# Add if using Platform detection
crux_platform = { git = "https://github.com/redbadger/crux", branch = "master" }
```

When Crux 0.17 is published to crates.io, replace git dependencies with versioned ones:

```toml
crux_core = "0.17"
crux_http = "0.16"   # check actual published version
```

## Shared Crate `Cargo.toml`

### Minimal (Render only)

```toml
[package]
name = "shared"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[lib]
crate-type = ["cdylib", "lib", "staticlib"]

[[bin]]
name = "codegen"
required-features = ["codegen"]

[features]
uniffi = ["dep:uniffi"]
wasm_bindgen = ["dep:wasm-bindgen", "getrandom/wasm_js"]
codegen = [
    "crux_core/cli",
    "dep:clap",
    "dep:log",
    "dep:pretty_env_logger",
    "uniffi",
]
facet_typegen = ["crux_core/facet_typegen"]

[dependencies]
crux_core.workspace = true
serde = { workspace = true, features = ["derive"] }
facet.workspace = true

# optional dependencies
clap = { version = "4", optional = true, features = ["derive"] }
getrandom = { version = "0.3", optional = true, default-features = false }
log = { version = "0.4", optional = true }
pretty_env_logger = { version = "0.5", optional = true }
uniffi = { version = "0.29", optional = true }
wasm-bindgen = { version = "0.2", optional = true }
```

### With HTTP capability

Add to `[dependencies]`:

```toml
crux_http.workspace = true
url = "2"
```

Add to `[features]`:

```toml
codegen = [
    "crux_core/cli",
    "crux_http/facet_typegen",    # add typegen for each capability
    "dep:clap",
    "dep:log",
    "dep:pretty_env_logger",
    "uniffi",
]
facet_typegen = [
    "crux_core/facet_typegen",
    "crux_http/facet_typegen",
]
```

### With Key-Value capability

Add to `[dependencies]`:

```toml
crux_kv.workspace = true
serde_json = "1.0"              # for serializing values to bytes
```

Add typegen features:

```toml
codegen = [
    "crux_core/cli",
    "crux_kv/facet_typegen",
    "dep:clap",
    "dep:log",
    "dep:pretty_env_logger",
    "uniffi",
]
facet_typegen = [
    "crux_core/facet_typegen",
    "crux_kv/facet_typegen",
]
```

### With custom SSE capability

Add to `[dependencies]`:

```toml
async-sse = "5"
async-std = "1"
futures = "0.3"
```

### Dev dependencies

```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

## `crate-type` Explained

The `shared` crate compiles to three library types:

| Type | Purpose | Used by |
|------|---------|---------|
| `lib` | Standard Rust library | Rust shells, `cargo test`, codegen binary |
| `staticlib` | Static library (`.a`) | iOS (linked into Swift app via Xcode) |
| `cdylib` | C-ABI dynamic library (`.so`/`.dylib`) | Android (loaded via JNA in Kotlin) |

## `rust-toolchain.toml`

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

This ensures the correct toolchain and cross-compilation targets are available.
Targets can be trimmed if not all platforms are needed.

## Feature Flags

| Feature | Purpose | When to enable |
|---------|---------|----------------|
| `uniffi` | UniFFI bindings for iOS/Android | Building for native mobile |
| `wasm_bindgen` | wasm-bindgen for Web | Building for WASM |
| `codegen` | Type generation CLI | Running `cargo run --bin codegen` |
| `facet_typegen` | Facet-based type generation | Used by codegen feature |

During normal development and testing, no features need to be enabled.
`cargo test` and `cargo check` work with default features.

## Codegen Binary

The `codegen` binary generates foreign language types. It replaces the old
`shared_types` crate approach.

The binary is declared in `Cargo.toml`:

```toml
[[bin]]
name = "codegen"
required-features = ["codegen"]
```

Run it with:

```bash
cargo run --bin codegen --features codegen
```

The binary source is typically auto-generated by the `crux_core/cli` feature
and does not need a separate source file.

## `.gitignore`

```
/target
Cargo.lock
```

Include `Cargo.lock` in `.gitignore` for library crates (the standard Rust convention).
If the project is an application (has binary targets other than codegen), keep `Cargo.lock`
tracked instead.

## Complete Example: Workspace with HTTP + KV

```toml
[workspace]
members = ["shared"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
crux_core = { git = "https://github.com/redbadger/crux", branch = "master" }
crux_http = { git = "https://github.com/redbadger/crux", branch = "master" }
crux_kv = { git = "https://github.com/redbadger/crux", branch = "master" }
serde = "1.0"
serde_json = "1.0"
facet = "=0.31"
```
