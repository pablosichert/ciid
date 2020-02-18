#[test]
fn test_verify_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir("./tests/files")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        }
    }

    for file in files {
        let output = std::process::Command::new("./target/debug/ciid")
            .arg("--verify-name")
            .arg(file)
            .output()?;

        assert!(output.status.success());
    }

    Ok(())
}
