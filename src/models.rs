use std::path::PathBuf;

pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub extension: Option<String>,
}

#[derive(Default)]
pub struct Stats {
    pub files_moved: usize,
    pub folders_created: usize,
    pub errors: usize,
}

pub type OrganizeResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
