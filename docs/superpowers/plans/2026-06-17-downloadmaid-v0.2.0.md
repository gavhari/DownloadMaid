# DownloadMaid v0.2.0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add config file support, recursive scanning, and glob-pattern blacklist to DownloadMaid.

**Architecture:** Refactor single-file main.rs into focused modules (config, scanner, organizer, cli, models). Load config from ~/.config/downloadmaid/config.toml with CLI override. Use walkdir for recursion, glob for blacklist.

**Tech Stack:** Rust, toml crate, walkdir crate, glob crate, dirs crate

## Global Constraints

- Language: Rust
- Config format: TOML
- Config location: ~/.config/downloadmaid/config.toml
- Dependencies: toml 0.8, walkdir 2, glob 0.3, dirs 5
- CLI args override config values
- Recursive scanning default: true
- Grouping: per extension (lowercase)
- Files without extension → others/ folder

---

### Task 1: Add Dependencies and Create Module Structure

**Files:**
- Modify: `Cargo.toml`
- Create: `src/models.rs`
- Create: `src/config.rs`
- Create: `src/cli.rs`
- Create: `src/scanner.rs`
- Create: `src/organizer.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: Empty module stubs with `pub use` in main.rs

- [ ] **Step 1: Add dependencies to Cargo.toml**

```toml
[dependencies]
toml = "0.8"
walkdir = "2"
glob = "0.3"
dirs = "5"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create src/models.rs with shared types**

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub extension: Option<String>,
}

#[derive(Default, Debug)]
pub struct Stats {
    pub files_moved: usize,
    pub folders_created: usize,
    pub errors: usize,
}
```

- [ ] **Step 3: Create empty module stubs**

Create `src/config.rs`:
```rust
// Config loading and merging
```

Create `src/cli.rs`:
```rust
// CLI argument parsing
```

Create `src/scanner.rs`:
```rust
// Directory scanning
```

Create `src/organizer.rs`:
```rust
// File organization logic
```

- [ ] **Step 4: Update main.rs to declare modules**

At top of `src/main.rs`:
```rust
mod models;
mod config;
mod cli;
mod scanner;
mod organizer;

use models::{FileEntry, Stats};
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully with new deps

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/models.rs src/config.rs src/cli.rs src/scanner.rs src/organizer.rs src/main.rs
git commit -m "refactor: add dependencies and module structure"
```

---

### Task 2: Implement Config Module

**Files:**
- Modify: `src/config.rs`
- Create: `tests/test_config.rs`

**Interfaces:**
- Consumes: `dirs` crate for XDG paths, `toml` crate for parsing
- Produces: `AppConfig` struct, `load_config() -> AppConfig`

- [ ] **Step 1: Write failing test for default config**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.path.ends_with("Downloads"));
        assert_eq!(config.recursive, true);
        assert_eq!(config.dry_run, false);
        assert_eq!(config.blacklist.len(), 0);
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test test_default_config`
Expected: FAIL with "AppConfig not found"

- [ ] **Step 3: Implement AppConfig struct and default**

```rust
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_path")]
    pub path: PathBuf,
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub blacklist: Vec<String>,
}

fn default_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to determine home directory")
        .join("Downloads")
}

fn default_recursive() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            recursive: true,
            dry_run: false,
            blacklist: Vec::new(),
        }
    }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test test_default_config`
Expected: PASS

- [ ] **Step 5: Write test for config file parsing**

```rust
#[test]
fn test_parse_config_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"
path = "/custom/path"
recursive = false
blacklist = ["*.tmp", "*.cache"]
"#).unwrap();

    let config = parse_config_file(file.path()).unwrap();
    assert_eq!(config.path, PathBuf::from("/custom/path"));
    assert_eq!(config.recursive, false);
    assert_eq!(config.blacklist, vec!["*.tmp", "*.cache"]);
}
```

- [ ] **Step 6: Implement parse_config_file**

```rust
use std::path::Path;
use std::fs;

pub fn parse_config_file(path: &Path) -> Option<AppConfig> {
    let contents = fs::read_to_string(path).ok()?;
    toml::from_str(&contents).ok()
}
```

- [ ] **Step 7: Run test**

Run: `cargo test test_parse_config_file`
Expected: PASS

- [ ] **Step 8: Implement load_config with XDG path**

```rust
pub fn load_config() -> AppConfig {
    let mut config = AppConfig::default();
    
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("downloadmaid").join("config.toml");
        if let Some(file_config) = parse_config_file(&config_path) {
            config = file_config;
        }
    }
    
    config
}
```

- [ ] **Step 9: Commit**

```bash
git add src/config.rs
git commit -m "feat: implement config loading from TOML"
```

---

### Task 3: Implement CLI Module

**Files:**
- Modify: `src/cli.rs`

**Interfaces:**
- Produces: `CliArgs` struct, `parse_args() -> CliArgs`

- [ ] **Step 1: Define CliArgs struct**

```rust
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct CliArgs {
    pub path: Option<PathBuf>,
    pub dry_run: Option<bool>,
    pub recursive: Option<bool>,
}
```

- [ ] **Step 2: Implement parse_args**

```rust
use std::env;

pub fn parse_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();
    let mut cli_args = CliArgs::default();
    
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dry-run" => cli_args.dry_run = Some(true),
            "--no-recursive" => cli_args.recursive = Some(false),
            path_arg if !path_arg.starts_with('-') => {
                cli_args.path = Some(PathBuf::from(path_arg));
            }
            _ => {}
        }
    }
    
    cli_args
}
```

- [ ] **Step 3: Add merge function to AppConfig**

In `src/config.rs`:
```rust
impl AppConfig {
    pub fn merge_cli_args(mut self, cli: crate::cli::CliArgs) -> Self {
        if let Some(path) = cli.path {
            self.path = path;
        }
        if let Some(dry_run) = cli.dry_run {
            self.dry_run = dry_run;
        }
        if let Some(recursive) = cli.recursive {
            self.recursive = recursive;
        }
        self
    }
}
```

- [ ] **Step 4: Test parse_args manually**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs src/config.rs
git commit -m "feat: add CLI argument parsing and merge logic"
```

---

### Task 4: Extract Organizer Module

**Files:**
- Modify: `src/organizer.rs`
- Modify: `src/main.rs` (extract functions)

**Interfaces:**
- Consumes: `FileEntry`, `Stats` from models
- Produces: `organize_files()`, `determine_folder()`, `move_file()`, `get_unique_dest_path()`

- [ ] **Step 1: Move organize_files to organizer.rs**

Copy from main.rs to `src/organizer.rs`:
```rust
use std::path::{Path, PathBuf};
use std::fs;
use crate::models::{FileEntry, Stats};

type OrganizeResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn organize_files(
    entries: Vec<FileEntry>,
    base_path: &Path,
    dry_run: bool,
) -> OrganizeResult<Stats> {
    let mut stats = Stats::default();

    for entry in entries {
        let folder = determine_folder(entry.extension.clone());
        let dest_dir = base_path.join(&folder);

        if !dest_dir.exists() {
            if dry_run {
                println!("[dry-run] Would create folder: {}/", folder);
            } else {
                if let Err(e) = fs::create_dir(&dest_dir) {
                    eprintln!("Error creating folder {}: {}", folder, e);
                    stats.errors += 1;
                    continue;
                }
            }
            stats.folders_created += 1;
        }

        let dest_file = get_unique_dest_path(&entry.path, &dest_dir);

        if dry_run {
            println!(
                "[dry-run] {} → {}/{}",
                entry.name,
                folder,
                dest_file.file_name().unwrap().to_str().unwrap()
            );
        } else {
            match move_file(&entry.path, &dest_file) {
                Ok(_) => stats.files_moved += 1,
                Err(e) => {
                    eprintln!("Error moving {}: {}", entry.name, e);
                    stats.errors += 1;
                }
            }
        }
    }

    Ok(stats)
}

fn determine_folder(extension: Option<String>) -> String {
    extension.unwrap_or_else(|| "others".to_string())
}

fn move_file(src: &Path, dst: &Path) -> OrganizeResult<()> {
    if fs::rename(src, dst).is_err() {
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

fn get_unique_dest_path(src: &Path, dest_dir: &Path) -> PathBuf {
    let file_name = src.file_name().unwrap().to_str().unwrap();
    let mut dest = dest_dir.join(file_name);
    let mut counter = 1;

    while dest.exists() {
        let stem = src.file_stem().unwrap().to_str().unwrap();
        let ext = src
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_default();
        dest = dest_dir.join(format!("{}({}){}", stem, counter, ext));
        counter += 1;
    }

    dest
}
```

- [ ] **Step 2: Remove extracted functions from main.rs**

Delete `organize_files`, `determine_folder`, `move_file`, `get_unique_dest_path`, `OrganizeResult` from main.rs.

Keep tests in main.rs for now (will migrate later).

- [ ] **Step 3: Update main.rs to use organizer module**

In main.rs:
```rust
use organizer::organize_files;
```

- [ ] **Step 4: Verify compilation and tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/organizer.rs src/main.rs
git commit -m "refactor: extract organizer module from main"
```

---

### Task 5: Implement Scanner Module with Blacklist

**Files:**
- Modify: `src/scanner.rs`
- Modify: `src/main.rs` (extract scan_directory)

**Interfaces:**
- Consumes: `glob` crate, `FileEntry` from models
- Produces: `scan_directory()`, `matches_blacklist()`

- [ ] **Step 1: Write test for blacklist matching**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_blacklist() {
        let patterns = vec!["*.tmp".to_string(), "temp_*".to_string()];
        
        assert!(matches_blacklist("file.tmp", &patterns));
        assert!(matches_blacklist("temp_data.txt", &patterns));
        assert!(!matches_blacklist("report.pdf", &patterns));
    }
}
```

- [ ] **Step 2: Implement matches_blacklist**

```rust
use glob::Pattern;

pub fn matches_blacklist(filename: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .ok()
            .map(|p| p.matches(filename))
            .unwrap_or(false)
    })
}
```

- [ ] **Step 3: Run test**

Run: `cargo test test_matches_blacklist`
Expected: PASS

- [ ] **Step 4: Extract and modify scan_directory from main.rs**

```rust
use std::path::Path;
use std::fs;
use crate::models::FileEntry;

pub fn scan_directory(
    path: &Path,
    recursive: bool,
    blacklist: &[String],
) -> Result<Vec<FileEntry>, std::io::Error> {
    if recursive {
        scan_recursive(path, blacklist)
    } else {
        scan_flat(path, blacklist)
    }
}

fn scan_flat(path: &Path, blacklist: &[String]) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() || file_type.is_symlink() {
            continue;
        }

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();

        if name.starts_with('.') || matches_blacklist(&name, blacklist) {
            continue;
        }

        let extension = get_extension(&name);

        entries.push(FileEntry {
            name: name.clone(),
            path: entry.path(),
            extension,
        });
    }

    Ok(entries)
}

fn scan_recursive(path: &Path, blacklist: &[String]) -> Result<Vec<FileEntry>, std::io::Error> {
    use walkdir::WalkDir;
    let mut entries = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('.') || matches_blacklist(&file_name, blacklist) {
            continue;
        }

        let extension = get_extension(&file_name);

        entries.push(FileEntry {
            name: file_name.clone(),
            path: entry.path().to_path_buf(),
            extension,
        });
    }

    Ok(entries)
}

fn get_extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit('.')
        .next()
        .filter(|&ext| ext != file_name && !ext.is_empty())
        .map(|ext| ext.to_lowercase())
}
```

- [ ] **Step 5: Update main.rs to use scanner module**

Remove `scan_directory` and `get_extension` from main.rs.

Add: `use scanner::scan_directory;`

- [ ] **Step 6: Run tests**

Run: `cargo test`
Expected: All pass

- [ ] **Step 7: Commit**

```bash
git add src/scanner.rs src/main.rs
git commit -m "feat: implement scanner with recursive and blacklist support"
```

---

### Task 6: Wire Everything Together in Main

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: All modules (config, cli, scanner, organizer)
- Produces: Updated main() function

- [ ] **Step 1: Rewrite main function**

```rust
fn main() {
    // Load config from file
    let config = config::load_config();
    
    // Parse CLI args and merge
    let cli_args = cli::parse_args();
    let config = config.merge_cli_args(cli_args);
    
    // Validate path
    if !config.path.exists() {
        eprintln!("Error: Path does not exist: {}", config.path.display());
        std::process::exit(1);
    }
    
    let prefix = if config.dry_run { "[dry-run] " } else { "" };
    println!(
        "{}DownloadMaid — Organizing {}",
        prefix,
        config.path.display()
    );
    println!();
    
    // Scan directory
    let entries = match scanner::scan_directory(&config.path, config.recursive, &config.blacklist) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    };
    
    // Organize files
    let stats = match organizer::organize_files(entries, &config.path, config.dry_run) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error organizing files: {}", e);
            std::process::exit(1);
        }
    };
    
    // Print summary
    println!();
    println!(
        "{}Done: {} files processed, {} folders created, {} errors.",
        prefix, stats.files_moved, stats.folders_created, stats.errors
    );
}
```

- [ ] **Step 2: Test with dry-run**

Run: `cargo run -- --dry-run`
Expected: Runs without error

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire config, cli, scanner, organizer in main"
```

---

### Task 7: Add Integration Tests

**Files:**
- Create: `tests/test_recursive.rs`
- Create: `tests/test_blacklist.rs`
- Modify: `tests/integration_test.rs`

**Interfaces:**
- Tests full v0.2.0 workflow

- [ ] **Step 1: Create recursive test**

```rust
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_recursive_organization() {
    let test_dir = PathBuf::from("/tmp/downloadmaid_recursive_test");
    
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir(&test_dir).unwrap();
    
    // Create nested structure
    fs::create_dir(test_dir.join("subdir1")).unwrap();
    fs::create_dir(test_dir.join("subdir1/subdir2")).unwrap();
    
    File::create(test_dir.join("root.pdf")).unwrap();
    File::create(test_dir.join("subdir1/nested.jpg")).unwrap();
    File::create(test_dir.join("subdir1/subdir2/deep.zip")).unwrap();
    
    // Run with recursive (default)
    let output = Command::new("cargo")
        .args(&["run", "--", test_dir.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Verify flattened structure
    assert!(test_dir.join("pdf/root.pdf").exists());
    assert!(test_dir.join("jpg/nested.jpg").exists());
    assert!(test_dir.join("zip/deep.zip").exists());
    
    fs::remove_dir_all(&test_dir).unwrap();
}
```

- [ ] **Step 2: Create blacklist test**

```rust
#[test]
fn test_blacklist_filtering() {
    let test_dir = PathBuf::from("/tmp/downloadmaid_blacklist_test");
    
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir(&test_dir).unwrap();
    
    File::create(test_dir.join("keep.pdf")).unwrap();
    File::create(test_dir.join("skip.tmp")).unwrap();
    File::create(test_dir.join("temp_file.txt")).unwrap();
    File::create(test_dir.join("normal.txt")).unwrap();
    
    // Create config
    let config_dir = test_dir.join(".config/downloadmaid");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
blacklist = ["*.tmp", "temp_*"]
"#,
    )
    .unwrap();
    
    // Set XDG_CONFIG_HOME
    std::env::set_var("XDG_CONFIG_HOME", test_dir.join(".config"));
    
    let output = Command::new("cargo")
        .args(&["run", "--", test_dir.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Verify blacklisted files remain
    assert!(test_dir.join("skip.tmp").exists());
    assert!(test_dir.join("temp_file.txt").exists());
    
    // Verify non-blacklisted moved
    assert!(test_dir.join("pdf/keep.pdf").exists());
    assert!(test_dir.join("txt/normal.txt").exists());
    
    fs::remove_dir_all(&test_dir).unwrap();
}
```

- [ ] **Step 3: Update existing integration test**

Ensure `tests/integration_test.rs` still works with new module structure.

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add tests/
git commit -m "test: add recursive and blacklist integration tests"
```

---

### Task 8: Update README and Documentation

**Files:**
- Modify: `README.md`

**Interfaces:**
- Documents v0.2.0 features

- [ ] **Step 1: Update README with v0.2.0 features**

Add to Features section:
```markdown
- **Config file support** — persistent settings via `~/.config/downloadmaid/config.toml`
- **Recursive scanning** — scans subdirectories and flattens to top-level folders
- **Blacklist patterns** — skip files matching glob patterns (e.g., `*.tmp`, `temp_*`)
```

Add Configuration section:
```markdown
## Configuration

Create `~/.config/downloadmaid/config.toml`:

```toml
# Default target directory
path = "~/Downloads"

# Enable recursive scanning
recursive = true

# Glob patterns for files to skip
blacklist = [
    "*.tmp",
    "*.cache",
    "temp_*",
    ".DS_Store",
    "Thumbs.db",
]
```

CLI arguments override config file settings:
- `downloadmaid /custom/path` — override path
- `downloadmaid --no-recursive` — disable recursive scanning
- `downloadmaid --dry-run` — preview mode
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README with v0.2.0 features"
```

---

## Self-Review Checklist

**Spec coverage:**
- Config file (TOML) → Task 2 ✅
- Recursive scanning → Task 5 ✅
- Blacklist (glob patterns) → Task 5 ✅
- CLI override → Task 3 ✅
- Module structure → Task 1, 4, 5, 6 ✅
- Integration tests → Task 7 ✅
- Documentation → Task 8 ✅

**No placeholders:** All code blocks complete ✅
**Type consistency:** FileEntry, Stats, AppConfig used consistently ✅

All requirements covered.
