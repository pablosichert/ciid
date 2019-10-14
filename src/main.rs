use chrono::DateTime;
use clap::{App, Arg};
use sha2::Digest;

unsafe fn transmute_vec<Input, Output>(
    data: Vec<Input>,
) -> Result<Vec<Output>, Box<dyn std::error::Error>> {
    let size_input = std::mem::size_of::<Input>();
    let size_output = std::mem::size_of::<Output>();

    let factor = size_input / size_output;

    if size_output * factor != size_input {
        Err("Input size must be a multiple of output size")
    } else {
        Ok(())
    }?;

    let pointer = data.as_ptr() as *mut Output;
    let length = data.len();
    let capacity = data.capacity();

    std::mem::forget(data);

    let result = Vec::from_raw_parts(pointer, length * factor, capacity * factor);

    Ok(result)
}

fn sortable_base_16(data: &[u8]) -> String {
    let mut result = String::new();

    for byte in data {
        let upper = (('a' as u8) + (byte >> 4)) as char;
        let lower = (('a' as u8) + (byte & 0b00001111)) as char;

        result.push(upper);
        result.push(lower);
    }

    return result;
}

fn exiftool(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("exiftool")
        .args(args)
        .output()
        .map_err(|error| format!("Failed executing command: {}", error))?;

    let error = match output.status.code() {
        Some(code) if code == 0 => None,
        Some(code) => Some(format!("Process has terminated with code {}", code)),
        None => Some("Process has been terminated by signal".to_owned()),
    };

    let stderr = std::str::from_utf8(&output.stderr)
        .map_err(|error| format!("Failed encoding stderr as UTF-8: {}", error))?;

    match (error, stderr) {
        (None, "") => Ok(()),
        (None, stderr) => Err(format!("Stderr: {}", stderr)),
        (Some(error), "") => Err(error),
        (Some(error), stderr) => Err(format!("{}. Stderr: {}", error, stderr)),
    }?;

    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("Failed encoding stdout as UTF-8: {}", error))?;

    Ok(stdout.to_owned())
}

fn get_timestamp(file_path: &str) -> Result<[u8; 8], Box<dyn std::error::Error>> {
    let output = exiftool(&[
        "-p",
        r#"${dateTimeOriginal#;DateFmt("%Y-%m-%d %H:%M:%S")}.${subSecTimeOriginal} ${dateTimeOriginal#;DateFmt("%z")}"#,
        file_path
    ]).map_err(|error| format!("Failed running exiftool: {}", error))?;

    let timestamp = DateTime::parse_from_str(&output, "%Y-%m-%d %H:%M:%S%.f %z\n")
        .map_err(|error| format!("Failed parsing exiftool timestamp: {}", error))?;

    let timestamp = timestamp.timestamp_millis();
    let timestamp = unsafe { std::mem::transmute::<_, [u8; 8]>(timestamp.to_be()) };

    Ok(timestamp)
}

fn get_fingerprint(file_path: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let image = rawloader::decode_file(file_path)?;

    let data: Vec<u8> = match image.data {
        rawloader::RawImageData::Float(data) => unsafe { transmute_vec(data) },
        rawloader::RawImageData::Integer(data) => unsafe { transmute_vec(data) },
    }
    .map_err(|error| format!("Failed transmuting data: {}", error))?;

    let mut hasher = sha2::Sha256::new();
    hasher.input(data);

    let sha256 = hasher.result();

    let mut fingerprint: [u8; 32] = Default::default();
    fingerprint.copy_from_slice(&sha256);

    Ok(fingerprint)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("CIID - Chronological Image Identifier")
        .arg(Arg::with_name("file path").takes_value(true).required(true))
        .get_matches();

    let file_path = matches
        .value_of("file path")
        .ok_or("No file path provided")?;

    let timestamp = get_timestamp(file_path)
        .map_err(|error| format!("Failed generating timestamp data: {}", error))?;

    let fingerprint = get_fingerprint(file_path)
        .map_err(|error| format!("Failed generating fingerprint data: {}", error))?;

    println!(
        "{}-{}",
        sortable_base_16(&timestamp[2..]),
        sortable_base_16(&fingerprint)
    );

    Ok(())
}
