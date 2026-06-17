use std::env;
use std::path::{Path, PathBuf};
use std::fs;

struct Config {
    path: PathBuf,
    dry_run: bool,
}

struct FileEntry {
    name: String,
    path: PathBuf,
    extension: Option<String>,
}

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
    fn test_get_extension() {
        assert_eq!(get_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_extension("file.TXZ"), Some("txz".to_string()));
        assert_eq!(get_extension("file"), None);
        assert_eq!(get_extension(".hidden"), None);
        assert_eq!(get_extension("file.tar.gz"), Some("gz".to_string()));
    }
}