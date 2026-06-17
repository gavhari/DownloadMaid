# DownloadMaid — Design Spec

## Summary

CLI one-shot Rust program that organizes files in a Downloads folder by moving each file into a subfolder named after its extension. Files without an extension go to `others/`.

## Requirements

- Language: Rust
- Execution model: CLI one-shot (no daemon, no watcher)
- Grouping: per extension (`pdf/`, `jpg/`, `zip/`, etc.)
- Files without extension: moved to `others/`
- Default target: `~/Downloads`
- Support custom path via CLI argument
- Support `--dry-run` flag for preview without moving

## Architecture

Single binary, single source file (`src/main.rs`). No external crate dependencies — stdlib only.

### Functions

| Function | Purpose |
|---|---|
| `main()` | Parse CLI args, invoke `organize()`, print summary |
| `organize(path, dry_run) -> Result` | Core logic: read dir, group files, create folders, move files |
| `get_extension(file_name) -> Option<String>` | Extract lowercase extension from filename |

### Data Flow

```
Input:  ~/Downloads/report.pdf, photo.jpg, data.zip, README

Output: ~/Downloads/
        ├── pdf/report.pdf
        ├── jpg/photo.jpg
        ├── zip/data.zip
        └── others/README
```

### Behavior Rules

1. Only process regular files (skip directories, symlinks).
2. Skip hidden files (names starting with `.`).
3. Extensions are lowercased (`PDF` → `pdf/`).
4. Folders are created on-demand.
5. If destination file already exists, append a counter (`report(1).pdf`).
6. In `--dry-run` mode, print what would happen without moving.

### CLI Interface

```
downloadmaid                    # Organize ~/Downloads
downloadmaid /path/to/folder    # Organize custom folder
downloadmaid --dry-run          # Preview without moving
downloadmaid /path --dry-run    # Combined
```

Arguments parsed via `std::env::args` (no clap dependency).

### Error Handling

- Permission denied on a file → log warning, skip, continue.
- `fs::rename` fails (cross-device) → fallback to copy + delete.
- Invalid/nonexistent path → exit with error message and code 1.
- All errors use `Result<(), Box<dyn std::error::Error>>`.

### Output Format

```
DownloadMaid — Organizing /home/user/Downloads

  report.pdf        → pdf/report.pdf
  photo.jpg         → jpg/photo.jpg
  data.zip          → zip/data.zip
  README            → others/README

Done: 4 files moved, 4 folders created, 0 errors.
```

In dry-run mode, prefix output with `[dry-run]` and replace "moved" with "would be moved".

## Testing

Manual testing scenarios:
1. Various file types (pdf, jpg, zip, mp4, etc.)
2. Empty directory
3. Files without extension
4. Nested directories (should be skipped)
5. Hidden files (should be skipped)
6. Duplicate filenames in destination
7. `--dry-run` mode
8. Cross-device move (copy+delete fallback)
