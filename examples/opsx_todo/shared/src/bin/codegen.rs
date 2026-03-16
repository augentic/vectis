use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use crux_core::{
    cli::{BindgenArgsBuilder, bindgen},
    type_generation::facet::{Config, TypeRegistry},
};
use log::info;

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

fn main() -> anyhow::Result<()> {
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
