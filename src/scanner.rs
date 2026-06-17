use std::fs;
use std::path::Path;

use crate::models::FileEntry;

pub fn scan_directory(path: &Path) -> Result<Vec<FileEntry>, std::io::Error> {
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
