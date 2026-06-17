use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct Config {
    path: PathBuf,
    dry_run: bool,
}

struct FileEntry {
    name: String,
    path: PathBuf,
    extension: Option<String>,
}

#[derive(Default)]
struct Stats {
    files_moved: usize,
    folders_created: usize,
    errors: usize,
}

type OrganizeResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_default_downloads_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Downloads")
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    let mut path = get_default_downloads_path();
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

fn determine_folder(extension: Option<String>) -> String {
    extension.unwrap_or_else(|| "others".to_string())
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

fn move_file(src: &Path, dst: &Path) -> OrganizeResult<()> {
    if fs::rename(src, dst).is_err() {
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

fn organize_files(
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

fn main() {
    let config = match parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let prefix = if config.dry_run { "[dry-run] " } else { "" };
    println!(
        "{}DownloadMaid — Organizing {}",
        prefix,
        config.path.display()
    );
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
    println!(
        "{}Done: {} files processed, {} folders created, {} errors.",
        prefix, stats.files_moved, stats.folders_created, stats.errors
    );
}

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

fn get_extension(file_name: &str) -> Option<String> {
    if file_name.starts_with('.') && !file_name[1..].contains('.') {
        return None;
    }
    file_name
        .rsplit('.')
        .next()
        .filter(|&ext| ext != file_name && !ext.is_empty())
        .map(|ext| ext.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
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

    #[test]
    fn test_determine_destination_folder() {
        assert_eq!(determine_folder(Some("pdf".to_string())), "pdf");
        assert_eq!(determine_folder(None), "others");
        assert_eq!(determine_folder(Some("ZIP".to_string())), "ZIP");
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_extension("file.TXZ"), Some("txz".to_string()));
        assert_eq!(get_extension("file"), None);
        assert_eq!(get_extension(".hidden"), None);
        assert_eq!(get_extension("file.tar.gz"), Some("gz".to_string()));
    }

    #[test]
    fn test_unique_dest_path() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();

        let src = PathBuf::from("/somewhere/file.txt");

        assert_eq!(
            get_unique_dest_path(&src, dir).file_name().unwrap(),
            "file.txt"
        );

        File::create(dir.join("file.txt")).unwrap();
        assert_eq!(
            get_unique_dest_path(&src, dir).file_name().unwrap(),
            "file(1).txt"
        );
    }
}
