use clap::{App, Arg};
use chrono::DateTime;
use sha2::Digest;

#[derive(Debug)]
struct Error {
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for Error {}

unsafe fn transmute_vec<Input, Output>(
    data: Vec<Input>,
) -> Result<Vec<Output>, Box<dyn std::error::Error>> {
    let size_input = std::mem::size_of::<Input>();
    let size_output = std::mem::size_of::<Output>();

    let factor = size_input / size_output;

    if size_output * factor != size_input {
        return Err(Box::new(Error {
            message: "Input size must be a multiple of output size".to_owned(),
        }));
    }

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

fn get_timestamp(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("exiftool")
            .arg("-p")
            .arg(r#"${dateTimeOriginal#;DateFmt("%Y-%m-%d %H:%M:%S")}.${subSecTimeOriginal} ${dateTimeOriginal#;DateFmt("%z")}"#)
            .arg(file_path)
            .output()?;

    let stderr = std::str::from_utf8(&output.stderr)?;

    if stderr != "" {
        return Err(Box::new(Error {
            message: stderr.to_owned()
        }));
    }

    let stdout = std::str::from_utf8(&output.stdout)?;

    let mut timestamp = stdout.to_owned();
    timestamp.pop();

    let timestamp = DateTime::parse_from_str(&timestamp, "%Y-%m-%d %H:%M:%S%.f %z")?;
    let timestamp = timestamp.timestamp_millis();
    let timestamp = unsafe { std::mem::transmute::<_, [u8;8]>(timestamp.to_be()) };
    let timestamp = sortable_base_16(&timestamp[2..]);

    Ok(timestamp)
}

fn get_fingerprint(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let image = rawloader::decode_file(file_path)?;

    let data: Vec<u8> = match image.data {
        rawloader::RawImageData::Float(data) => unsafe { transmute_vec(data)? },
        rawloader::RawImageData::Integer(data) => unsafe { transmute_vec(data)? },
    };

    let mut hasher = sha2::Sha256::new();
    hasher.input(data);

    let sha256 = hasher.result();

    let identifier = sortable_base_16(&sha256);

    Ok(identifier)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("CIID - Chronological Image Identifier")
        .arg(Arg::with_name("file path").takes_value(true).required(true))
        .get_matches();

    let file_path = matches
        .value_of("file path")
        .ok_or("No file path provided")?;

    let timestamp = get_timestamp(file_path)?;
    let fingerprint = get_fingerprint(file_path)?;

    println!("{}-{}", timestamp, fingerprint);

    Ok(())
}
