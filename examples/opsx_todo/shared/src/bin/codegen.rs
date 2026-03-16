#[cfg(feature = "codegen")]
fn main() {
    crux_core::cli::run::<shared::TodoApp>().expect("codegen failed");
}

#[cfg(not(feature = "codegen"))]
fn main() {
    eprintln!("This binary requires the 'codegen' feature.");
}
