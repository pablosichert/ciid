# ciid - Chronological Image Identifier

`ciid` is a utility to derive a chronologically sortable, unique identifier for
images.

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
0A1B2C3D4E-5F6G7H8J9K0M1N2P3Q4R5S6T7V8W9X0Y1Z2A3B4C5D6E7F8G9H0J
└───┬────┘ └────────────────────────┬─────────────────────────┘
 timestamp                hash of image buffer
```

The first part of the identifier encodes the creation date of the image (a
50-bit timestamp with millisecond-precision), while the second part is a hash
(SHA-256) based on the contents of the image buffer.

The encoding uses
[Douglas Crackford's base32](https://www.crockford.com/base32.html) alphabet
with the following characters:

`0`, `1`, `2`, `3`, `4`, `5`, `6`, `7`, `8`, `9`, `A`, `B`, `C`, `D`, `E`, `F`,
`G`, `H`, `J`, `K`, `M`, `N`, `P`, `Q`, `R`, `S`, `T`, `V`, `W`, `X`, `Y`, `Z`.

Following criteria were considered when choosing the character set:

- encode information sufficiently compact (in this case 5 bits per character)
- have sensible alphabetical ordering on file systems (timestamps with a higher
  value should appear strictly after lower ones)
- no distinction between upper- und lowercase, avoiding issues on case
  insensitive file systems
- be safe to use in URLs

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [exiftool](https://github.com/exiftool/exiftool)
- [LibRaw](https://github.com/LibRaw/LibRaw)

For help with installing the dependencies, have a look at the
[Dockerfile](https://github.com/pablosichert/ciid/blob/master/Dockerfile).

Install the `ciid` binary onto your system via
[`cargo`](https://doc.rust-lang.org/cargo/commands/cargo-install.html):

```bash
$ cargo install ciid
```

## Usage

```bash
$ ciid [FLAGS] <file path>
```

## Flags

| Short | Long          | Description                                                              |
| ----- | ------------- | ------------------------------------------------------------------------ |
| -h    | --help        | Prints help information                                                  |
|       | --rename-file | Renames the file to the derived identifier. Preserves the file extension |
| -V    | --version     | Prints version information                                               |
|       | --verify-name | Verifies if the provided file name is equal to the derived identifier    |

## Arguments

| Name          | Description        |
| ------------- | ------------------ |
| \<file path\> | Path to image file |

## Prior Art

The timestamp used in `ciid` was inspired by [ulid](https://github.com/ulid/spec).
