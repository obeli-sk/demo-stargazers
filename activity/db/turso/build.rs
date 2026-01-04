use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=wit/");
    Opts {
        generate_all: true,
        additional_derive_attributes: vec!["PartialEq".to_string()],
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;
    Ok(())
}
