#[test]
fn test_print() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d\n"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_print_identifier() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--print")
        .arg("${identifier}")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_print_identifier_no_hash() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--no-hash")
        .arg("--print")
        .arg("${identifier}")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "01483617175960");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_print_date_time() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--print")
        .arg("${date_time}")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "2017-01-05T13:52:55.960+02:00"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_print_timestamp() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--print")
        .arg("${timestamp}")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "1483617175960");
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_timestamp_digits_less() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--timestamp-digits")
        .arg("0")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "1483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d\n"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_timestamp_digits_default() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d\n"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_timestamp_digits_more() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--timestamp-digits")
        .arg("20")
        .arg("./tests/files/01483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "00000001483617175960-d4f894f5e3414125ffa2b8a94637ab44034f9de931a6ca3ef6025c97cbd28a8d\n"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

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

        assert_eq!(std::str::from_utf8(&output.stderr)?, "");
        assert!(output.status.success());
    }

    Ok(())
}

#[test]
fn test_verify_name_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir("./tests/files")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        }
    }

    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--verify-name")
        .args(files)
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}
