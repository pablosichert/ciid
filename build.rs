fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-flags=-l raw");

    let bindings = bindgen::Builder::default()
        .detect_include_paths(true)
        .header("src/libraw.h")
        .generate()
        .map_err(|error| format!("Failed generating bindings: {:?}", error))?;

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    bindings
        .write_to_file(out_path.join("libraw.rs"))
        .map_err(|error| format!("Failed writing bindings: {}", error))?;

    Ok(())
}
