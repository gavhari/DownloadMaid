mod models;
mod config;
mod cli;
mod scanner;
mod organizer;

use scanner::scan_directory;
use organizer::organize_files;

fn main() {
    let cli_args = cli::parse_args();

    let mut config = config::load_config();
    config = config.merge_cli_args(cli_args);

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
