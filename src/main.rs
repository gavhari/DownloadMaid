use std::env;
use std::path::PathBuf;

struct Config {
    path: PathBuf,
    dry_run: bool,
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