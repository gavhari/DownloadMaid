# DownloadMaid Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a CLI one-shot Rust program that organizes files in Downloads folder by extension.

**Architecture:** Single binary (`src/main.rs`), stdlib only, parse args → scan directory → move files per extension → print summary.

**Tech Stack:** Rust (no external crates)

## Global Constraints

- Language: Rust
- Dependencies: stdlib only (no crates)
- Default path: `~/Downloads`
- Grouping: per extension (lowercase folder names)
- Files without extension → `others/` folder
- Skip: directories, hidden files (`.` prefix)
- Handle cross-device moves: fallback to copy + delete
- Duplicate filenames: append counter (`file(1).ext`)

---

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs` (stub)
- Create: `.gitignore`

**Interfaces:**
- Produces: Compilable Rust binary skeleton

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "downloadmaid"
version = "0.1.0"
edition = "2021"

[bin]
name = "downloadmaid"
path = "src/main.rs"
```

- [ ] **Step 2: Create .gitignore**

```
/target
**/*.rs.bk
Cargo.lock
```

- [ ] **Step 3: Create src/main.rs stub**

```rust
fn main() {
    println!("DownloadMaid — Placeholder");
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: `Finished dev [unoptimized]` with binary at `target/debug/downloadmaid`

- [ ] **Step 5: Test binary runs**

Run: `./target/debug/downloadmaid`
Expected: `DownloadMaid — Placeholder`

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml .gitignore src/main.rs
git commit -m "chore: initialize Rust project"
```

---

### Task 2: CLI Argument Parsing

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `std::env::args`
- Produces: `Config` struct with `path: PathBuf`, `dry_run: bool`

- [ ] **Step 1: Add Config struct and parse function**

```rust
use std::env;
use std::path::PathBuf;

struct Config {
    path: PathBuf,
    dry_run: bool,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    
    let mut path = dirs::home_dir()
        .expect("Failed to determine home directory")
        .join("Downloads");
    let mut dry_run = false;
    
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            path_arg => {
                if !path_arg.starts_with('-') {
                    path = PathBuf::from(path_arg);
                }
            }
        }
    }
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    Ok(Config { path, dry_run })
}
```

- [ ] **Step 2: Update main to use Config**

```rust
fn main() {
    let config = match parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("Target: {}", config.path.display());
    println!("Dry run: {}", config.dry_run);
}
```

- [ ] **Step 3: Add dirs dependency temporarily**

Actually, let me fix this — we need a pure stdlib solution:

```rust
fn get_default_downloads_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Downloads")
}
```

And update parse_args to use this instead of `dirs::home_dir()`.

- [ ] **Step 4: Test argument parsing**

Run: `cargo build && ./target/debug/downloadmaid`
Expected: `Target: /home/<user>/Downloads` and `Dry run: false`

Run: `./target/debug/downloadmaid /tmp/test`
Expected: `Target: /tmp/test`

Run: `./target/debug/downloadmaid --dry-run`
Expected: `Dry run: true`

- [ ] **Step 5: Test error handling**

Run: `./target/debug/downloadmaid /nonexistent`
Expected: `Error: Path does not exist: /nonexistent`

- [ ] **Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: add CLI argument parsing"
```

---

### Task 3: Directory Scanning

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `Config` from Task 2
- Produces: `scan_directory(path: &Path) -> Vec<FileEntry>` where `FileEntry` has `name: String`, `path: PathBuf`, `extension: Option<String>`

- [ ] **Step 1: Add FileEntry struct**

```rust
struct FileEntry {
    name: String,
    path: PathBuf,
    extension: Option<String>,
}
```

- [ ] **Step 2: Write failing test for scan_directory**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_scan_directory_skips_hidden() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        File::create(temp_path.join(".hidden")).unwrap();
        File::create(temp_path.join("visible.txt")).unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();
        
        let entries = scan_directory(temp_path).unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible.txt");
    }
}
```

- [ ] **Step 3: Run test to verify failure**

Run: `cargo test`
Expected: `error[E0425]: cannot find function 'scan_directory' in this scope`

- [ ] **Step 4: Implement scan_directory**

```rust
fn scan_directory(path: &Path) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        
        // Skip directories and symlinks
        if file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy().to_string();
        
        // Skip hidden files
        if name.starts_with('.') {
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
```

- [ ] **Step 5: Add get_extension helper**

```rust
fn get_extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit('.')
        .next()
        .filter(|&ext| ext != file_name && !ext.is_empty())
        .map(|ext| ext.to_lowercase())
}
```

- [ ] **Step 6: Run test to verify pass**

Run: `cargo test`
Expected: `test test_scan_directory_skips_hidden ... ok`

- [ ] **Step 7: Add test for extension extraction**

```rust
#[test]
fn test_get_extension() {
    assert_eq!(get_extension("file.txt"), Some("txt".to_string()));
    assert_eq!(get_extension("file.TXZ"), Some("txz".to_string()));
    assert_eq!(get_extension("file"), None);
    assert_eq!(get_extension(".hidden"), None);
    assert_eq!(get_extension("file.tar.gz"), Some("gz".to_string()));
}
```

- [ ] **Step 8: Run tests**

Run: `cargo test`
Expected: All pass

- [ ] **Step 9: Commit**

```bash
git add src/main.rs
git commit -m "feat: add directory scanning and extension extraction"
```

---

### Task 4: File Organization Logic

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `FileEntry` vector from Task 3
- Produces: `organize_files(entries: Vec<FileEntry>, base_path: &Path, dry_run: bool) -> Result<Stats, Error>`

- [ ] **Step 1: Add Stats and Error structs**

```rust
#[derive(Default)]
struct Stats {
    files_moved: usize,
    folders_created: usize,
    errors: usize,
}

#[derive(Debug)]
struct OrganizeError {
    file_name: String,
    message: String,
}

impl std::error::Error for OrganizeError {}

impl std::fmt::Display for OrganizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.file_name, self.message)
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

- [ ] **Step 2: Write failing test for folder grouping**

```rust
#[test]
fn test_determine_destination_folder() {
    assert_eq!(determine_folder(Some("pdf".to_string())), "pdf");
    assert_eq!(determine_folder(None), "others");
    assert_eq!(determine_folder(Some("ZIP".to_string())), "zip");
}
```

- [ ] **Step 3: Run test to verify failure**

Run: `cargo test`
Expected: `cannot find function 'determine_folder'`

- [ ] **Step 4: Implement determine_folder**

```rust
fn determine_folder(extension: Option<String>) -> String {
    extension.unwrap_or_else(|| "others".to_string())
}
```

- [ ] **Step 5: Run test to verify pass**

Run: `cargo test test_determine_destination_folder`
Expected: PASS

- [ ] **Step 6: Implement organize_files (skeleton)**

```rust
fn organize_files(
    entries: Vec<FileEntry>,
    base_path: &Path,
    dry_run: bool,
) -> Result<Stats> {
    let mut stats = Stats::default();
    
    for entry in entries {
        let folder = determine_folder(entry.extension);
        let dest_dir = base_path.join(&folder);
        
        // Create folder if needed
        if !dest_dir.exists() {
            if dry_run {
                println!("[dry-run] Would create folder: {}/", folder);
            } else {
                fs::create_dir(&dest_dir)?;
            }
            stats.folders_created += 1;
        }
        
        // Determine destination file path
        let dest_file = dest_dir.join(&entry.name);
        
        if dry_run {
            println!("[dry-run] {} → {}/{}", entry.name, folder, entry.name);
        } else {
            move_file(&entry.path, &dest_file)?;
        }
        
        stats.files_moved += 1;
    }
    
    Ok(stats)
}
```

- [ ] **Step 7: Implement move_file with fallback**

```rust
fn move_file(src: &Path, dst: &Path) -> Result<()> {
    if fs::rename(src, dst).is_err() {
        // Cross-device move fallback: copy + delete
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}
```

- [ ] **Step 8: Update main to call organize_files**

```rust
fn main() {
    let config = match parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    let prefix = if config.dry_run { "[dry-run] " } else { "" };
    println!("{}DownloadMaid — Organizing {}", prefix, config.path.display());
    println!();
    
    let entries = match scan_directory(&config.path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    };
    
    let stats = match organize_files(entries, &config.path, config.dry_run) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error organizing files: {}", e);
            std::process::exit(1);
        }
    };
    
    println!();
    println!("{}Done: {} files processed, {} folders created, {} errors.",
        prefix, stats.files_moved, stats.folders_created, stats.errors);
}
```

- [ ] **Step 9: Test with real directory**

Run: `mkdir -p /tmp/test_organize && echo "test" > /tmp/test_organize/file.txt && echo "test2" > /tmp/test_organize/doc.pdf`

Run: `./target/debug/downloadmaid /tmp/test_organize --dry-run`
Expected: Shows `[dry-run]` prefix and would‑be actions

- [ ] **Step 10: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement file organization logic"
```

---

### Task 5: Handle Duplicate Filenames

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: Destination path from Task 4
- Produces: `get_unique_dest_path(src: &Path, dest_dir: &Path) -> PathBuf`

- [ ] **Step 1: Write failing test for duplicate handling**

```rust
#[test]
fn test_unique_dest_path() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path();
    
    File::create(dir.join("file.txt")).unwrap();
    
    // First call returns original name
    let src = PathBuf::from("/somewhere/file.txt");
    assert_eq!(
        get_unique_dest_path(&src, dir).file_name().unwrap(),
        "file.txt"
    );
    
    // Second call adds counter
    File::create(dir.join("file.txt")).unwrap();
    assert_eq!(
        get_unique_dest_path(&src, dir).file_name().unwrap(),
        "file(1).txt"
    );
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test`
Expected: `cannot find function 'get_unique_dest_path'`

- [ ] **Step 3: Implement get_unique_dest_path**

```rust
fn get_unique_dest_path(src: &Path, dest_dir: &Path) -> PathBuf {
    let file_name = src.file_name().unwrap().to_str().unwrap();
    let mut dest = dest_dir.join(file_name);
    let mut counter = 1;
    
    while dest.exists() {
        let stem = src.file_stem().unwrap().to_str().unwrap();
        let ext = src.extension().and_then(|s| s.to_str()).map(|s| format!(".{}", s)).unwrap_or_default();
        dest = dest_dir.join(format!("{}({}){}", stem, counter, ext));
        counter += 1;
    }
    
    dest
}
```

- [ ] **Step 4: Update organize_files to use unique paths**

Replace `let dest_file = dest_dir.join(&entry.name);` with:
```rust
let dest_file = get_unique_dest_path(&entry.path, &dest_dir);
```

- [ ] **Step 5: Run test to verify pass**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 6: Test with actual duplicates**

Run: 
```bash
mkdir -p /tmp/dup_test
echo "a" > /tmp/dup_test/file.txt
echo "b" > /tmp/dup_test/file.txt
cp /tmp/dup_test/file.txt /tmp/dup_test/file2.txt
./target/debug/downloadmaid /tmp/dup_test
ls /tmp/dup_test/txt/
```

Expected: `file.txt` and `file(1).txt`

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "feat: handle duplicate filenames with counter"
```

---

### Task 6: Error Handling and Logging

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: Errors from file operations
- Produces: Per‑file error logging without stopping entire process

- [ ] **Step 1: Update organize_files to handle individual errors**

```rust
fn organize_files(
    entries: Vec<FileEntry>,
    base_path: &Path,
    dry_run: bool,
) -> Result<Stats> {
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
            println!("[dry-run] {} → {}/{}", entry.name, folder, dest_file.file_name().unwrap().to_str().unwrap());
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
```

- [ ] **Step 2: Test with permission error scenario**

Run: 
```bash
mkdir -p /tmp/perm_test/subfolder
chmod 000 /tmp/perm_test/subfolder
echo "test" > /tmp/perm_test/file.txt
./target/debug/downloadmaid /tmp/perm_test
```

Expected: Error logged, program continues

- [ ] **Step 3: Test dry-run doesn't create folders**

Run: 
```bash
mkdir -p /tmp/dry_test
echo "test" > /tmp/dry_test/file.txt
./target/debug/downloadmaid /tmp/dry_test --dry-run
ls /tmp/dry_test/
```

Expected: No folders created, file still in place

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add per-file error handling and logging"
```

---

### Task 7: Integration Test

**Files:**
- Modify: `src/main.rs` (add integration test)
- Create: `tests/integration_test.rs`

**Interfaces:**
- Tests full workflow end‑to‑end

- [ ] **Step 1: Create integration test file**

```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_full_workflow() {
    let test_dir = PathBuf::from("/tmp/downloadmaid_test");
    
    // Cleanup and setup
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir(&test_dir).unwrap();
    
    // Create test files
    File::create(test_dir.join("doc.pdf")).unwrap().write_all(b"pdf").unwrap();
    File::create(test_dir.join("photo.jpg")).unwrap().write_all(b"jpg").unwrap();
    File::create(test_dir.join("archive.zip")).unwrap().write_all(b"zip").unwrap();
    File::create(test_dir.join("README")).unwrap().write_all(b"readme").unwrap();
    File::create(test_dir.join(".hidden")).unwrap().write_all(b"hidden").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();
    
    // Run binary
    let output = Command::new("cargo")
        .args(&["run", "--", test_dir.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Verify structure
    assert!(test_dir.join("pdf/doc.pdf").exists());
    assert!(test_dir.join("jpg/photo.jpg").exists());
    assert!(test_dir.join("zip/archive.zip").exists());
    assert!(test_dir.join("others/README").exists());
    assert!(!test_dir.join(".hidden").exists()); // Should be skipped
    assert!(test_dir.join("subdir").exists()); // Should remain as dir
    
    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}
```

- [ ] **Step 2: Add tempfile dev dependency**

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run integration test**

Run: `cargo test --test integration_test`

Expected: PASS

- [ ] **Step 4: Test dry-run integration**

```bash
cargo run -- /tmp/dry_test --dry-run
```

Expected: No files moved, `[dry-run]` prefix visible

- [ ] **Step 5: Commit**

```bash
git add src/main.rs Cargo.toml tests/
git commit -m "test: add integration test"
```

---

## Plan Review

**Spec coverage checklist:**
- CLI one-shot → Task 2 (parse_args)
- Per extension grouping → Task 4 (determine_folder)
- Files without extension → Task 4 (others folder)
- Hidden files skipped → Task 3 (scan_directory filter)
- Duplicates handled → Task 5 (get_unique_dest_path)
- Dry-run mode → Task 2, 4, 6 (config flag throughout)
- Error handling → Task 6 (per‑file error logging)
- Cross-device fallback → Task 4 (move_file fallback)

All spec requirements covered.

---

## Execution

Plan saved to `docs/superpowers/plans/2026-06-17-downloadmaid.md`.

Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
