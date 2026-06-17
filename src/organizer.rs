use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{FileEntry, OrganizeResult, Stats};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_determine_folder() {
        assert_eq!(super::determine_folder(Some("pdf".to_string())), "pdf");
        assert_eq!(super::determine_folder(None), "others");
        assert_eq!(super::determine_folder(Some("ZIP".to_string())), "ZIP");
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
