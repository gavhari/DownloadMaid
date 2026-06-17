# DownloadMaid

A lightweight CLI tool that automatically organizes files in your Downloads folder by grouping them into subfolders based on file extension.

## Features

- **Extension-based organization** — groups files into folders named after their extension (pdf/, jpg/, zip/, etc.)
- **Dry-run mode** — preview changes before applying them
- **Duplicate handling** — automatically renames duplicates with counters (file(1).pdf, file(2).pdf)
- **Error resilience** — logs errors for individual files but continues processing
- **Cross-device support** — handles moves across different filesystems
- **Hidden file filtering** — skips hidden files (starting with `.`)
- **Zero dependencies** — uses only Rust stdlib

## Installation

### From Source

```bash
git clone git@github.com:gavhari/DownloadMaid.git
cd DownloadMaid
cargo build --release
```

The binary will be at `target/release/downloadmaid`.

### Install Locally

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Organize ~/Downloads (default)
downloadmaid

# Organize a custom folder
downloadmaid /path/to/folder

# Preview changes without moving files
downloadmaid --dry-run

# Preview changes in custom folder
downloadmaid /path/to/folder --dry-run
```

### Example

Before:
```
Downloads/
├── report.pdf
├── photo.jpg
├── data.zip
└── README
```

After running `downloadmaid`:
```
Downloads/
├── pdf/
│   └── report.pdf
├── jpg/
│   └── photo.jpg
├── zip/
│   └── data.zip
└── others/
    └── README
```

### Behavior

- **Regular files only** — skips directories and symlinks
- **Hidden files skipped** — files starting with `.` are ignored
- **Extensions lowercase** — `PDF` becomes `pdf/`
- **No extension → `others/`** — files without extensions go to `others/` folder
- **Duplicate handling** — if `file.pdf` exists in `pdf/`, the next one becomes `file(1).pdf`
- **Cross-device moves** — automatically falls back to copy+delete when needed
- **Error handling** — logs errors for individual files, continues with others

## Development

### Running Tests

```bash
cargo test
```

### Project Structure

```
src/
└── main.rs          # Single-file implementation
tests/
└── integration_test.rs
docs/
├── superpowers/
│   ├── specs/       # Design specifications
│   └── plans/       # Implementation plans
```

## License

MIT

## Contributing

Issues and pull requests welcome at https://github.com/gavhari/DownloadMaid
