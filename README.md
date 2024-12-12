<h1 align="center">
  ciid - Chronological Image Identifier
</h1>

<p align="center">
  <code>ciid</code> is a utility to derive a chronologically sortable, unique
  identifier for images.
  <br />
  <br/>
  <a href="https://github.com/pablosichert/ciid/actions">
    <img
      alt="ciid build status"
      src="https://img.shields.io/github/workflow/status/pablosichert/ciid/CI"
    />
  </a>
  <a href="https://crates.io/crates/ciid">
    <img
      alt="ciid on crates.io"
      src="https://img.shields.io/crates/v/ciid.svg"
    />
  </a>
</p>

Usually, digital cameras and phones assign file names to images with a sequence
of only 4 digits (e.g. `IMG_1234.dng`). Those names will easily clash for any
sufficiently large amount of images.

`ciid` tackles this problem by deriving a hash from the image buffer.
Additionally to being able to derive an identifier that is very unlikely to
clash, this hash can later be used to check the integrity of the image content.

Some image processing programs update metadata of files (e.g inline JPEG-
previews, tags, modified date). The resulting `ciid` will be unaffected from
those changes, since only the actual image buffer is hashed. This has the nice
side-effect that proprietary camera RAW file formats and converted `.dng` files
will yield the same identifier most of the time.

Here's how a resulting identifier looks like:

```
01234567890123-a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1
└─────┬──────┘ └──────────────────────────────┬───────────────────────────────┘
  timestamp                         hash of image buffer
```

The first part of the identifier encodes the creation date of the image (a unix
timestamp with millisecond precision), while the second part is a hash (SHA-256)
based on the contents of the image buffer.

Following criteria were considered when choosing the identifier:

- have sensible alphabetical ordering on file systems (timestamps with a higher
  value should appear strictly after lower ones)
- encode information sufficiently compact
- be safe to use in URLs

## Installation (via script)

Download and run the installation script:

```bash
$ curl -s https://raw.githubusercontent.com/pablosichert/ciid/master/bin/install.sh | bash
```

## Installation (manually)

### Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [exiftool](https://github.com/exiftool/exiftool)
- [LibRaw](https://github.com/LibRaw/LibRaw)

For help with installing the dependencies, have a look at the
[install script](https://github.com/pablosichert/ciid/blob/master/bin/install.sh).

Install the `ciid` binary onto your system via
[`cargo`](https://doc.rust-lang.org/cargo/commands/cargo-install.html):

```bash
$ cargo install ciid
```

## Usage

```bash
$ ciid [FLAGS] <file path>...
```

## Flags

| Short | Long          | Description                                                                                  |
| ----- | ------------- | -------------------------------------------------------------------------------------------- |
| -h    | --help        | Prints help information                                                                      |
|       | --no-hash     | If provided, the raw image will not be hashed, and no hash will be appended to the file name |
|       | --rename-file | Renames the file to the derived identifier. Preserves the file extension                     |
| -V    | --version     | Prints version information                                                                   |
|       | --verify-name | Verifies if the provided file name is equal to the derived identifier                        |

## Options

| Short | Long                                    | Description                                                                                                                                                    |
| ----- | --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|       | --print \<template\>                    | Prints provided template to stdout, substituting variables with file information. Available variables: ${file_path}, ${identifier}, ${date_time}, ${timestamp} |
|       | --timestamp-digits \<timestamp digits\> | Minimum number of digits the timestamp should carry. Will be padded with zeros from the left                                                                   |

## Arguments

| Name             | Description        |
| ---------------- | ------------------ |
| \<file path\>... | Path to image file |

## FAQ

#### Why not use a more human-readable format for the timestamp?

Why do we encode the timestamp as `01234567890123` instead of e.g.
`2009-02-13 23:31:30.123`? The timestamp represents an
unambiguous<a href="#footnote-leap-seconds"><sup>1</sup></a> single point in
time, whereas the date string needs to be contextualized with a time zone. That
means that you would either need to annotate the date string with a time zone or
change the file name every time you are on a system which uses a different time
zone.

Apart from that, the former encoding is more compact.

While unfortunately it's not easy to derive the actual date from the timestamp
just by looking at it, you can compare two timestamps chronologically by sorting
them by value.

<sup id="footnote-leap-seconds">1</sup> ignoring
[leap-seconds](https://en.wikipedia.org/wiki/Leap_second).

## Changelog

The changelog format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### [0.2.0]

#### Changed

- The timestamp used in the identifier is no longer a byte sequence encoded
  using base32. Instead, it uses the plain decimal representation of a unix
  timestamp with millisecond precision. The reason for this change is that the
  file system UI (at least on macOS) does not sort file names strictly
  chronologically. E.g. files with names `01`, `0a`, `10`, `a0` should preserve
  the order, but get presented in order `0a`, `01`, `10`, `a0`.
- The hash used in the identifier is now encoded using lowercase hex instead of
  base32, in accordance with how common tools encode SHA-256 hashes.

#### Added

- The new CLI option `--timestamp-digits` can be used to control how many digits
  the timestamp should carry at a minimum. Missing zeros will be padded from the
  left.
