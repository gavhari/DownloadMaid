use downloadmaid::config::{parse_config_file, AppConfig};
use std::path::PathBuf;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert!(config.path.ends_with("Downloads"));
    assert_eq!(config.recursive, true);
    assert_eq!(config.dry_run, false);
    assert_eq!(config.blacklist.len(), 0);
}

#[test]
fn test_parse_config_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
path = "/custom/path"
recursive = false
blacklist = ["*.tmp", "*.cache"]
"#
    )
    .unwrap();

    let config = parse_config_file(file.path()).unwrap();
    assert_eq!(config.path, PathBuf::from("/custom/path"));
    assert_eq!(config.recursive, false);
    assert_eq!(config.blacklist, vec!["*.tmp", "*.cache"]);
}

#[test]
fn test_parse_config_file_missing_returns_none() {
    let config = parse_config_file(std::path::Path::new("/nonexistent/path/config.toml"));
    assert!(config.is_none());
}

#[test]
fn test_parse_config_file_partial_uses_defaults() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"recursive = false"#).unwrap();

    let config = parse_config_file(file.path()).unwrap();
    assert_eq!(config.recursive, false);
    assert!(config.path.ends_with("Downloads"));
    assert_eq!(config.dry_run, false);
    assert_eq!(config.blacklist.len(), 0);
}

#[test]
fn test_parse_config_file_dry_run() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"dry_run = true"#).unwrap();

    let config = parse_config_file(file.path()).unwrap();
    assert_eq!(config.dry_run, true);
}
