mod libraw;

use chrono::{DateTime, FixedOffset};
use clap::{App, Arg};
use image;
use regex;
use sha2::Digest;
use std::convert::TryInto;

/// Based on Douglas Crockford's base32 alphabet: https://www.crockford.com/base32.html.
const ALPHABET: &'static str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Encodes a timestamp to a string.
///
/// The byte representation of the timestamp is padded with zeros on the left if the amount of bits
/// needed to represent it can not exactly be encoded with the given encoding.
///
/// # Arguments
/// * `encoding` – Encoding used to encode the timestamp.
/// * `timestamp` – The timestamp to be encoded.
fn encode_timestamp(
    encoding: &data_encoding::Encoding,
    timestamp: &DateTime<FixedOffset>,
) -> Result<String, Box<dyn std::error::Error>> {
    let millis: u64 = timestamp
        .timestamp_millis()
        .try_into()
        .map_err(|_| "Timestamps before 1970-01-01T00:00:00Z are not supported")?;

    let bytes = millis.to_be_bytes().to_vec();

    // Pad bytes width zeros on the left
    let bytes = {
        let chunk_size = encoding.bit_width();
        let num_bytes = bytes.len();
        let num_chunks = num_bytes / chunk_size + (if num_bytes % chunk_size == 0 { 0 } else { 1 });
        let num_padded = (num_chunks * chunk_size) - num_bytes;

        let mut padded = vec![0; num_padded];
        padded.extend(bytes);
        padded
    };

    Ok(encoding.encode(&bytes))
}

/// Calls the `exiftool` command line tool and returns the contents of stdout.
///
/// # Arguments
/// * `args` – Command line arguments to be passed to `exiftool`.
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

/// Get the timestamp of a file based on EXIF-data.
///
/// # Arguments
/// * `file_path` – Path to file for which the timestamp should be read and returned.
fn get_timestamp(
    file_path: &std::path::Path,
) -> Result<DateTime<FixedOffset>, Box<dyn std::error::Error>> {
    let path = file_path.to_str().ok_or_else(|| "Invalid file path")?;

    let output = exiftool(&[
        "-p",
        r#"${dateTimeOriginal#;DateFmt("%Y-%m-%d %H:%M:%S")}.${subSecTimeOriginal} ${dateTimeOriginal#;DateFmt("%z")}"#,
        path
    ]).map_err(|error| format!("Failed running exiftool: {}", error))?;

    let timestamp = DateTime::parse_from_str(&output, "%Y-%m-%d %H:%M:%S%.f %z\n")
        .map_err(|error| format!("Failed parsing exiftool timestamp: {}", error))?;

    Ok(timestamp)
}

fn hash_image_jpeg(
    file_path: &std::path::Path,
    hasher: &mut sha2::Sha256,
) -> Result<(), Box<dyn std::error::Error>> {
    let image =
        image::open(file_path).map_err(|error| format!("Failed opening JPEG image: {}", error))?;

    let data = image.to_bytes();

    hasher.input(data);

    Ok(())
}

fn hash_image_raw(
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

/// Derive a hash for an image file. For the hash, only data contained in the image buffer is
/// considered, disregarding any metadata.
///
/// # Arguments
/// * `file_path` – Path to file for which the hash should be derived.
fn hash_image(file_path: &std::path::Path) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut hasher = sha2::Sha256::new();

    let extension = file_path
        .extension()
        .and_then(|extension| extension.to_str());

    match extension {
        Some(extension) if regex::Regex::new("(?i)jpe?g")?.is_match(extension) => {
            hash_image_jpeg(file_path, &mut hasher)
        }
        _ => hash_image_raw(file_path, &mut hasher),
    }
    .map_err(|error| {
        format!(
            "Failed hashing {} file: {}",
            extension.map_or_else(
                || "<no extension>".to_owned(),
                |extension| format!(".{}", extension)
            ),
            error
        )
    })?;

    let sha256 = hasher.result();

    let mut hash: [u8; 32] = Default::default();
    hash.copy_from_slice(&sha256);

    Ok(hash)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ciid - Chronological Image Identifier")
        .version(clap::crate_version!())
        .about(&*("\n".to_owned() + clap::crate_description!()))
        .arg(
            Arg::with_name("file path")
                .takes_value(true)
                .required(true)
                .help("Path to image file"),
        )
        .arg(
            Arg::with_name("verify name")
                .long("--verify-name")
                .help("Verifies if the provided file name is equal to the derived identifier"),
        )
        .arg(
            Arg::with_name("rename file")
                .long("--rename-file")
                .help("Renames the file to the derived identifier. Preserves the file extension"),
        )
        .get_matches();

    let file_path = matches
        .value_of("file path")
        .ok_or("No file path provided")?;

    let file_path = std::path::Path::new(file_path)
        .canonicalize()
        .map_err(|error| format!("Invalid file path: {}", error))?;

    let timestamp = get_timestamp(&file_path)
        .map_err(|error| format!("Failed deriving timestamp data: {}", error))?;

    let hash =
        hash_image(&file_path).map_err(|error| format!("Failed deriving image hash: {}", error))?;

    let encoding = {
        let mut spec = data_encoding::Specification::new();
        spec.symbols.push_str(ALPHABET);
        spec.encoding()
    }?;

    let timestamp_encoded = encode_timestamp(&encoding, &timestamp)?;

    let identifier = format!(
        "{}-{}",
        &timestamp_encoded[timestamp_encoded.len() - 10..],
        encoding.encode(&hash),
    );

    let verify_name = matches.is_present("verify name");
    let rename_file = matches.is_present("rename file");

    let hash_file_path = {
        let mut path = match file_path.parent() {
            Some(parent) => parent.into(),
            None => std::path::PathBuf::new(),
        };

        path.push(identifier.clone());

        if let Some(extension) = file_path.extension() {
            path.set_extension(extension);
        }

        path
    };

    if verify_name {
        if file_path != hash_file_path {
            Err(format!(
                r#"File name mismatch: Expected "{:?}", got "{:?}""#,
                hash_file_path, file_path
            ))?;
        }
    } else if rename_file {
        std::fs::rename(file_path.clone(), hash_file_path)?;
    } else {
        println!("{}", identifier);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_encode_timestamp {
        ($test_name:ident, $input:literal, $expected:literal) => {
            #[test]
            fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
                let encoding = {
                    let mut spec = data_encoding::Specification::new();
                    spec.symbols.push_str(ALPHABET);
                    spec.encoding()
                }?;

                let timestamp = DateTime::parse_from_str($input, "%Y-%m-%d %H:%M:%S%.f %z\n")?;

                assert_eq!($expected, encode_timestamp(&encoding, &timestamp)?);

                Ok(())
            }
        };
    }

    test_encode_timestamp!(
        test_encode_timestamp_unix_time,
        "1970-1-1 00:00:00.000 +0000",
        "0000000000000000"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_millisecond,
        "1970-1-1 00:00:00.001 +0000",
        "0000000000000001"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_32_milliseconds,
        "1970-1-1 00:00:00.032 +0000",
        "0000000000000010"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_second,
        "1970-1-1 00:00:01.000 +0000",
        "00000000000000Z8"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_minute,
        "1970-1-1 00:01:00.000 +0000",
        "0000000000001TK0"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_hour,
        "1970-1-1 01:00:00.000 +0000",
        "000000000003DVM0"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_day,
        "1970-1-2 00:00:00.000 +0000",
        "00000000002JCQ00"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_month,
        "1970-2-1 00:00:00.000 +0000",
        "0000000002FTA900"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_plus_1_year,
        "1971-1-1 00:00:00.000 +0000",
        "000000000XBV2B00"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_tz_minus_1,
        "1969-12-31 23:00:00.000 -0100",
        "0000000000000000"
    );

    test_encode_timestamp!(
        test_encode_timestamp_unix_time_tz_plus_1,
        "1970-1-1 01:00:00.000 +0100",
        "0000000000000000"
    );
}
