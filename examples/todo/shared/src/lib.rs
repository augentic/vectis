mod app;
pub mod ffi;
pub mod sse;

pub use app::*;
pub use crux_core::Core;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
