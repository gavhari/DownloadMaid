# DownloadMaid v0.2.0 — Design Spec

## Summary

Add config file support, recursive directory scanning, and glob-pattern blacklist to DownloadMaid. Restructure single-file implementation into focused modules.

## Requirements

- Config format: TOML
- Config location: `~/.config/downloadmaid/config.toml`
- CLI args override config values
- Recursive scanning (on/off via config, default on)
- Blacklist via glob patterns (matched against filename)
- Dependencies: `toml`, `walkdir`, `glob`, `dirs`

## Architecture

### Module Structure

```
src/
├── main.rs       # Entry point, CLI parsing dispatch
├── cli.rs        # CLI argument parsing
├── config.rs     # Config struct, TOML loading, merge logic
├── scanner.rs    # Directory scanning (flat + recursive)
├── organizer.rs  # File grouping, moving, duplicate handling
└── models.rs     # Shared types: FileEntry, Stats, Config
```

### Dependencies (Cargo.toml)

```toml
[dependencies]
toml = "0.8"
walkdir = "2"
glob = "0.3"
dirs = "5"
```

## Components

### config.rs

**Config struct:**
```rust
pub struct AppConfig {
    pub path: PathBuf,           // Default: ~/Downloads
    pub dry_run: bool,           // Default: false
    pub recursive: bool,         // Default: true
    pub blacklist: Vec<String>,  // Default: empty
}
```

**Loading order (later wins):**
1. Built-in defaults
2. `~/.config/downloadmaid/config.toml` (if exists)
3. CLI args

**Functions:**
- `load_config() -> AppConfig` — loads defaults + file + applies CLI
- `parse_config_file(path: &Path) -> Option<PartialConfig>` — TOML parsing
- `merge(defaults, file, cli) -> AppConfig` — merge layers

### scanner.rs

**Functions:**
- `scan_directory(path: &Path, recursive: bool, blacklist: &[String]) -> Vec<FileEntry>`
- `scan_flat(path: &Path, blacklist: &[String]) -> Vec<FileEntry>` — current behavior
- `scan_recursive(path: &Path, blacklist: &[String]) -> Vec<FileEntry>` — walkdir
- `matches_blacklist(filename: &str, patterns: &[String]) -> bool` — glob matching

### organizer.rs

Existing logic from v0.1.0, extracted:
- `organize_files(entries, base_path, dry_run) -> Stats`
- `determine_folder(extension) -> String`
- `move_file(src, dst) -> Result<()>`
- `get_unique_dest_path(src, dest_dir) -> PathBuf`

### cli.rs

- `parse_args() -> CliArgs`
- `CliArgs { path: Option<PathBuf>, dry_run: Option<bool> }`

### models.rs

- `FileEntry { name, path, extension }`
- `Stats { files_moved, folders_created, errors }`

## Config File Format

```toml
# ~/.config/downloadmaid/config.toml

path = "~/Downloads"         # Default target directory
recursive = true             # Recursive scanning
dry_run = false              # Default dry-run mode

# Glob patterns for files to skip
blacklist = [
    "*.tmp",
    "*.cache",
    "temp_*",
    ".DS_Store",
    "Thumbs.db",
]
```

## Data Flow

1. Load defaults (`~/Downloads`, recursive=true, no blacklist)
2. Read config file if exists → override defaults
3. Parse CLI args → override config
4. Scan directory (flat or recursive based on config)
5. Filter via blacklist
6. Organize files into extension folders
7. Print summary

## Error Handling

- Config file parse error → log warning, use defaults
- Config file not found → silent, use defaults
- Blacklist pattern invalid → log warning, skip that pattern
- Scanner errors → log per-file, continue (same as v0.1.0)

## Testing

- Unit tests for config loading and merging
- Unit tests for blacklist matching
- Unit tests for recursive vs flat scanning
- Integration test for full recursive workflow
- Integration test for blacklist filtering
