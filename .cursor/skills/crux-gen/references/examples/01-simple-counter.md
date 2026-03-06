# Example: Simple Counter (Render Only)

A minimal Crux app with local state and no external side-effects.
Demonstrates the basic App trait, Model, Event, ViewModel, Effect, and testing patterns.

## Capabilities Used

- **Render** (built-in)

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

## `shared/Cargo.toml`

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

clap = { version = "4", optional = true, features = ["derive"] }
getrandom = { version = "0.3", optional = true, default-features = false }
log = { version = "0.4", optional = true }
pretty_env_logger = { version = "0.5", optional = true }
uniffi = { version = "0.29", optional = true }
wasm-bindgen = { version = "0.2", optional = true }
```

## `shared/src/lib.rs`

```rust
mod app;
pub mod ffi;

pub use app::*;
pub use crux_core::Core;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
```

## `shared/src/app.rs`

```rust
use crux_core::{
    macros::effect,
    render::{render, RenderOperation},
    App, Command,
};
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Model {
    count: isize,
}

#[derive(Facet, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ViewModel {
    pub count: String,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    Increment,
    Decrement,
    Reset,
}

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
}

#[derive(Default)]
pub struct Counter;

impl App for Counter {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command {
        match event {
            Event::Increment => model.count += 1,
            Event::Decrement => model.count -= 1,
            Event::Reset => model.count = 0,
        }
        render()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            count: format!("Count is: {}", model.count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::App as _;

    #[test]
    fn initial_view_shows_zero() {
        let app = Counter;
        let model = Model::default();

        let view = app.view(&model);
        assert_eq!(view.count, "Count is: 0");
    }

    #[test]
    fn increment_updates_count() {
        let app = Counter;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Increment, &mut model);
        assert_eq!(model.count, 1);

        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.count, "Count is: 1");
    }

    #[test]
    fn decrement_updates_count() {
        let app = Counter;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Decrement, &mut model);
        assert_eq!(model.count, -1);

        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.count, "Count is: -1");
    }

    #[test]
    fn reset_sets_count_to_zero() {
        let app = Counter;
        let mut model = Model { count: 42 };

        let mut cmd = app.update(Event::Reset, &mut model);
        assert_eq!(model.count, 0);

        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.count, "Count is: 0");
    }

    #[test]
    fn sequence_of_events() {
        let app = Counter;
        let mut model = Model::default();

        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Increment, &mut model);
        let _ = app.update(Event::Decrement, &mut model);

        assert_eq!(model.count, 2);

        let view = app.view(&model);
        assert_eq!(view.count, "Count is: 2");
    }
}
```

## `shared/src/ffi.rs`

```rust
use crux_core::{
    Core,
    bridge::{Bridge, EffectId},
};

use crate::Counter;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm_bindgen", wasm_bindgen::prelude::wasm_bindgen)]
pub struct CoreFFI {
    core: Bridge<Counter>,
}

impl Default for CoreFFI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[cfg_attr(feature = "wasm_bindgen", wasm_bindgen::prelude::wasm_bindgen)]
impl CoreFFI {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    #[cfg_attr(
        feature = "wasm_bindgen",
        wasm_bindgen::prelude::wasm_bindgen(constructor)
    )]
    #[must_use]
    pub fn new() -> Self {
        Self {
            core: Bridge::new(Core::new()),
        }
    }

    #[must_use]
    pub fn update(&self, data: &[u8]) -> Vec<u8> {
        let mut effects = Vec::new();
        match self.core.update(data, &mut effects) {
            Ok(()) => effects,
            Err(e) => panic!("{e}"),
        }
    }

    #[must_use]
    pub fn resolve(&self, id: u32, data: &[u8]) -> Vec<u8> {
        let mut effects = Vec::new();
        match self.core.resolve(EffectId(id), data, &mut effects) {
            Ok(()) => effects,
            Err(e) => panic!("{e}"),
        }
    }

    #[must_use]
    pub fn view(&self) -> Vec<u8> {
        let mut view_model = Vec::new();
        match self.core.view(&mut view_model) {
            Ok(()) => view_model,
            Err(e) => panic!("{e}"),
        }
    }
}
```

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
