#![allow(clippy::cargo_common_metadata)]

mod app;
#[cfg(any(feature = "wasm_bindgen", feature = "uniffi"))]
mod ffi;
pub mod sse;

pub use app::*;
pub use crux_core::Core;

#[cfg(any(feature = "wasm_bindgen", feature = "uniffi"))]
pub use ffi::CoreFFI;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
