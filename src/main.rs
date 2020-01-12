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

fn fingerprint_image_jpeg(
    file_path: &std::path::Path,
    hasher: &mut sha2::Sha256,
) -> Result<(), Box<dyn std::error::Error>> {
    let image =
        image::open(file_path).map_err(|error| format!("Failed opening JPEG image: {}", error))?;

    let data = image.raw_pixels();

    hasher.input(data);

    Ok(())
}

fn fingerprint_image_raw(
    file_path: &std::path::Path,
    hasher: &mut sha2::Sha256,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = match file_path.to_str() {
        None => Err(format!("Invalid file path: {:?}", file_path)),
        Some(file_path) => Ok(file_path),
    }?;

    let file_path = std::ffi::CString::new(file_path)?;

    unsafe {
        let flags = 0;
        let data = libraw::libraw_init(flags);
        let error_code = libraw::libraw_open_file(data, file_path.as_ptr());

        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
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
                std::slice::from_raw_parts(raw_image, std::convert::TryInto::try_into(length)?);

            hasher.input(raw_buffer);

            Ok(())
        })();

        libraw::libraw_close(data);

        result
    }?;

    Ok(())
}

fn fingerprint_image(
    file_path: &std::path::Path,
    hasher: &mut sha2::Sha256,
) -> Result<(), Box<dyn std::error::Error>> {
    let extension = file_path
        .extension()
        .and_then(|extension| extension.to_str());

    match extension {
        Some(extension) if regex::Regex::new("(?i)jpe?g")?.is_match(extension) => {
            fingerprint_image_jpeg(file_path, hasher)
        }
        _ => fingerprint_image_raw(file_path, hasher),
    }
    .map_err(|error| {
        format!(
            "Failed fingerprinting {} file: {}",
            extension.map_or_else(
                || "<no extension>".to_owned(),
                |extension| format!(".{}", extension)
            ),
            error
        )
    })?;

    Ok(())
}

fn get_fingerprint(file_path: &std::path::Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut hasher = sha2::Sha256::new();

    fingerprint_image(file_path, &mut hasher)
        .map_err(|error| format!("Failed fingerprinting image: {}", error))?;

    let sha256 = hasher.result();

    let mut fingerprint: [u8; 32] = Default::default();
    fingerprint.copy_from_slice(&sha256);

    Ok(fingerprint)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("CIID - Chronological Image Identifier")
        .arg(Arg::with_name("file path").takes_value(true).required(true))
        .arg(
            Arg::with_name("rename file")
                .long("--rename-file")
                .help("Renames the file to the generated identifier. Preserves the file extension"),
        )
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

    let identifier = format!(
        "{}-{}",
        sortable_base_16(&timestamp[2..]),
        sortable_base_16(&fingerprint)
    );

    let rename_file = matches.is_present("rename file");

    if rename_file {
        let extension = file_path
            .extension()
            .and_then(|extension| extension.to_str());

        std::fs::rename(
            file_path.clone(),
            match extension {
                Some(extension) => format!("{}.{}", identifier, extension),
                None => identifier,
            },
        )?;
    } else {
        println!("{}", identifier);
    }

    Ok(())
}
