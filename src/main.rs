mod libraw;

use chrono::DateTime;
use clap::{App, Arg};
use image;
use regex;
use sha2::Digest;

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

fn get_timestamp(file_path: &std::path::Path) -> Result<[u8; 8], Box<dyn std::error::Error>> {
    let path = file_path.to_str().ok_or_else(|| "Invalid file path")?;

    let output = exiftool(&[
        "-p",
        r#"${dateTimeOriginal#;DateFmt("%Y-%m-%d %H:%M:%S")}.${subSecTimeOriginal} ${dateTimeOriginal#;DateFmt("%z")}"#,
        path
    ]).map_err(|error| format!("Failed running exiftool: {}", error))?;

    let timestamp = DateTime::parse_from_str(&output, "%Y-%m-%d %H:%M:%S%.f %z\n")
        .map_err(|error| format!("Failed parsing exiftool timestamp: {}", error))?;

    let timestamp = timestamp.timestamp_millis();
    let timestamp = unsafe { std::mem::transmute::<_, [u8; 8]>(timestamp.to_be()) };

    Ok(timestamp)
}

fn get_raw_image_data_from_jpeg(
    file_path: &std::path::Path,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let image =
        image::open(file_path).map_err(|error| format!("Failed opening JPEG image: {}", error))?;

    let data = image.raw_pixels();

    Ok(data)
}

fn get_raw_image_data_from_raw(
    file_path: &std::path::Path,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file_path = match file_path.to_str() {
        None => Err(format!("Invalid file path: {:?}", file_path)),
        Some(file_path) => Ok(file_path),
    }?;

    let file_path = std::ffi::CString::new(file_path)?;

    let data = unsafe {
        let flags = 0;
        let data = libraw::libraw_init(flags);
        let error_code = libraw::libraw_open_file(data, file_path.as_ptr());

        let result = (|| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            if error_code != 0 {
                Err(
                    std::ffi::CStr::from_ptr(libraw::libraw_strerror(error_code))
                        .to_string_lossy()
                        .to_owned(),
                )
                .map_err(|error| format!("Failed opening file: {}", error))?;
            }

            let error_code = libraw::libraw_unpack(data);

            if error_code != 0 {
                Err(
                    std::ffi::CStr::from_ptr(libraw::libraw_strerror(error_code))
                        .to_string_lossy()
                        .to_owned(),
                )?;
            }

            let data = match data.as_ref() {
                None => Err("Unexpected null pointer in LibRaw data")?,
                Some(pointer) => pointer,
            };

            let raw_image = data.rawdata.raw_image as *mut u8;

            if raw_image.is_null() {
                Err("Unexpected null pointer in LibRaw data.rawdata.raw_image")?;
            }

            let length = data.rawdata.sizes.raw_pitch * (data.rawdata.sizes.raw_height as u32);

            let raw_buffer =
                std::slice::from_raw_parts(raw_image, std::convert::TryInto::try_into(length)?)
                    .to_vec();

            Ok(raw_buffer)
        })();

        libraw::libraw_close(data);

        result
    }?;

    Ok(data)
}

fn get_raw_image_data(file_path: &std::path::Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let extension = file_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| {
            "Failed getting file extension, which is necessary to determine the file type"
        })?;

    let data = if regex::Regex::new("(?i)jpe?g")?.is_match(extension) {
        get_raw_image_data_from_jpeg(file_path)
    } else {
        get_raw_image_data_from_raw(file_path)
    }
    .map_err(|error| {
        format!(
            "Failed getting raw image data from .{} file: {}",
            extension, error
        )
    })?;

    Ok(data)
}

fn get_fingerprint(file_path: &std::path::Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let data = get_raw_image_data(file_path)
        .map_err(|error| format!("Failed getting raw image data: {}", error))?;

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

    let file_path = std::path::Path::new(file_path)
        .canonicalize()
        .map_err(|error| format!("Invalid file path: {}", error))?;

    let timestamp = get_timestamp(&file_path)
        .map_err(|error| format!("Failed generating timestamp data: {}", error))?;

    let fingerprint = get_fingerprint(&file_path)
        .map_err(|error| format!("Failed generating fingerprint data: {}", error))?;

    println!(
        "{}-{}",
        sortable_base_16(&timestamp[2..]),
        sortable_base_16(&fingerprint)
    );

    Ok(())
}
