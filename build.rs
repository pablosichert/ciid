fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-flags=-l raw");

    let bindings = bindgen::Builder::default()
        .detect_include_paths(true)
        .header("src/libraw.h")
        // For more information on the following blacklist see:
        // https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-316983630
        .blacklist_item("FP_NAN")
        .blacklist_item("FP_INFINITE")
        .blacklist_item("FP_ZERO")
        .blacklist_item("FP_SUBNORMAL")
        .blacklist_item("FP_NORMAL")
        .generate()
        .map_err(|error| format!("Failed generating bindings: {:?}", error))?;

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    bindings
        .write_to_file(out_path.join("libraw.rs"))
        .map_err(|error| format!("Failed writing bindings: {}", error))?;

    Ok(())
}
