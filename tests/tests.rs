#[test]
fn test_print() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("./tests/files/01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G\n"
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
        .arg("./tests/files/01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G.CR2")
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout)?,
        "01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G"
    );
    assert_eq!(std::str::from_utf8(&output.stderr)?, "");
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_print_date_time() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("./target/debug/ciid")
        .arg("--print")
        .arg("${date_time}")
        .arg("./tests/files/01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G.CR2")
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
        .arg("./tests/files/01B5Q7K2CR-TKW99XF3850JBZX2Q2MMCDXB8G1MZ7F966KCMFQP09E9FJYJHA6G.CR2")
        .output()?;

    assert_eq!(std::str::from_utf8(&output.stdout)?, "1483617175960");
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
