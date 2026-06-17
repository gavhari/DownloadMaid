use std::path::{Path, PathBuf};
use std::fs;

use toml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub path: PathBuf,
    pub recursive: bool,
    pub dry_run: bool,
    pub blacklist: Vec<String>,
    pub schedule: String,
}

fn default_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to determine home directory")
        .join("Downloads")
}

fn default_recursive() -> bool {
    true
}

fn default_schedule() -> String {
    "0 * * * *".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            recursive: default_recursive(),
            dry_run: false,
            blacklist: Vec::new(),
            schedule: default_schedule(),
        }
    }
}

pub fn parse_config_file(path: &Path) -> Option<AppConfig> {
    let contents = fs::read_to_string(path).ok()?;
    let toml_val: Value = toml::from_str(&contents).ok()?;

    let path_val = toml_val
        .get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(default_path);

    let recursive_val = toml_val
        .get("recursive")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(default_recursive);

    let dry_run_val = toml_val
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let blacklist_val = toml_val
        .get("blacklist")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);

    let schedule_val = toml_val
        .get("schedule")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(default_schedule);

    Some(AppConfig {
        path: path_val,
        recursive: recursive_val,
        dry_run: dry_run_val,
        blacklist: blacklist_val,
        schedule: schedule_val,
    })
}

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
