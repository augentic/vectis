use std::path::PathBuf;

use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum};
use crux_core::{
    cli::{BindgenArgsBuilder, bindgen},
    type_generation::facet::{Config, TypeRegistry},
};
use log::info;
use uniffi::deps::anyhow::Result;

use shared::TodoApp;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Language {
    Swift,
    Kotlin,
    Typescript,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    language: Language,
    #[arg(short, long)]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    let typegen_app = TypeRegistry::new().register_app::<TodoApp>()?.build()?;

    let name = match args.language {
        Language::Swift => "SharedTypes",
        Language::Kotlin => "com.vectis.todo",
        Language::Typescript => "shared_types",
    };
    let config = Config::builder(name, &args.output_dir)
        .add_extensions()
        .add_runtimes()
        .build();

    match args.language {
        Language::Swift => {
            info!("Typegen for Swift");
            typegen_app.swift(&config)?;

            info!("Bindgen for Swift (uniffi 0.31)");
            swift_bindgen(&args.output_dir)?;
        }
        Language::Kotlin => {
            info!("Typegen for Kotlin");
            typegen_app.kotlin(&config)?;

            info!("Bindgen for Kotlin");
            let bindgen_args = BindgenArgsBuilder::default()
                .crate_name(env!("CARGO_PKG_NAME").to_string())
                .kotlin(&args.output_dir)
                .build()?;
            bindgen(&bindgen_args)?;
        }
        Language::Typescript => {
            info!("Typegen for TypeScript");
            typegen_app.typescript(&config)?;
        }
    }

    Ok(())
}

/// Generate Swift UniFFI bindings using uniffi_bindgen 0.31 directly.
///
/// crux_core::cli::bindgen bundles uniffi_bindgen 0.29 which produces
/// symbol names incompatible with the uniffi 0.31 proc-macro scaffolding.
/// Using the matched version ensures the generated headers, modulemap,
/// and Swift source reference the correct C symbol names.
fn swift_bindgen(out_dir: &PathBuf) -> Result<()> {
    use cargo_metadata::MetadataCommand;
    use uniffi::{SwiftBindingsOptions, generate_swift_bindings};

    let metadata = MetadataCommand::new().no_deps().exec()?;
    let target_dir = &metadata.target_directory;

    let library_path = ["rlib", "dylib", "a"]
        .iter()
        .map(|ext| target_dir.join(format!("debug/libshared.{ext}")))
        .find(|p| p.exists())
        .ok_or_else(|| {
            uniffi::deps::anyhow::anyhow!(
                "compiled library not found in {target_dir}/debug/ — \
                 run `cargo build --features uniffi` first"
            )
        })?;

    let options = SwiftBindingsOptions {
        generate_swift_sources: true,
        generate_headers: true,
        generate_modulemap: true,
        source: library_path,
        out_dir: Utf8PathBuf::from_path_buf(out_dir.clone())
            .map_err(|p| uniffi::deps::anyhow::anyhow!("non-UTF8 path: {}", p.display()))?,
        ..SwiftBindingsOptions::default()
    };

    generate_swift_bindings(options)?;
    Ok(())
}
