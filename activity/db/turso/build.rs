use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=wit/");
    // `embed_migrations!` does not notice added files.
    println!("cargo:rerun-if-changed=../migrations");
    Opts {
        generate_all: true,
        additional_derive_attributes: vec!["PartialEq".to_string()],
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;
    Ok(())
}
