# FolderMaid

Lightweight CLI tool that automatically organizes files by grouping them into extension-based subfolders. Written in Rust, zero non-essential dependencies.

## Features

- **Extension-based sorting** — files grouped into folders named after extension (`pdf/`, `jpg/`, `zip/`, etc.)
- **Configurable via TOML** — target path, recursion, blacklist patterns, schedule, dry-run mode
- **CLI args override config** — per-run overrides for path, dry-run, recursion
- **Dry-run mode** — preview changes before moving anything
- **Duplicate handling** — auto-renames with counters (`file(1).pdf`, `file(2).pdf`)
- **Blacklist** — exclude directories or file patterns from processing
- **Error resilience** — logs per-file errors, continues processing rest
- **Cross-device moves** — falls back to copy+delete when rename fails across filesystems
- **Hidden file filtering** — skips files starting with `.`
- **Docker support** — deploy as container with cron scheduling
- **Zero heavy dependencies** — only uses Rust stdlib, `toml`, `walkdir`, `glob`, `dirs`

## Installation

### From Source

```bash
git clone git@github.com:gavhari/FolderMaid.git
cd FolderMaid
cargo build --release
```

Binary at `target/release/foldermaid`.

### Install Locally

```bash
cargo install --path .
```

### Docker

```bash
docker build -t foldermaid .
docker run -d \
  -v ~/Downloads:/data \
  -v ~/.config/foldermaid:/config \
  foldermaid
```

See [Docker deployment docs](docs/superpowers/specs/2026-06-17-docker-deploy-design.md).

## Usage

### Basic

```bash
foldermaid                        # Organize ~/Downloads
foldermaid /path/to/folder        # Custom folder
foldermaid --dry-run              # Preview only
foldermaid /path --dry-run        # Preview on custom folder
foldermaid --no-recursive         # Flat mode, no subdir recursion
```

### Configuration

Optional config at `~/.config/foldermaid/config.toml`:

```toml
path = "/home/user/Downloads"
recursive = true
dry_run = false
schedule = "0 * * * *"

blacklist = [
    "node_modules",
    ".git",
    "*.tmp",
]
```

CLI flags override matching config fields at runtime.

### Example

Before:
```
Downloads/
├── report.pdf
├── photo.jpg
├── data.zip
└── README
```

After `foldermaid`:
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

| Rule | Detail |
|------|--------|
| Files only | Skips directories and symlinks |
| Hidden skipped | Files starting with `.` ignored |
| Lowercase ext | `PDF` → `pdf/`, `Tar.Gz` → `gz/` |
| No extension | Goes to `others/` |
| Duplicates | `file.pdf` exists → `file(1).pdf` |
| Cross-device | Auto copy+delete fallback |
| Errors | Logged per-file, rest continues |

## Development

### Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs        # Entry point, orchestration
├── lib.rs         # Module exports
├── cli.rs         # CLI argument parsing
├── config.rs      # TOML config loading and merge
├── models.rs      # Data types (FileEntry, Stats, OrganizeResult)
├── scanner.rs     # Filesystem scanning
└── organizer.rs   # File movement logic
tests/
└── integration_test.rs
docs/
└── superpowers/
    ├── specs/     # Design specifications
    └── plans/     # Implementation plans
Dockerfile         # Container build
```

## License

MIT

## Contributing

Issues and PRs welcome at https://github.com/gavhari/FolderMaid
