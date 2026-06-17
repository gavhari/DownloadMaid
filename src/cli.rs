use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct CliArgs {
    pub path: Option<PathBuf>,
    pub dry_run: Option<bool>,
    pub recursive: Option<bool>,
}

pub fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut cli_args = CliArgs::default();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dry-run" => cli_args.dry_run = Some(true),
            "--no-recursive" => cli_args.recursive = Some(false),
            path_arg if !path_arg.starts_with('-') => {
                cli_args.path = Some(PathBuf::from(path_arg));
            }
            _ => {}
        }
    }

    cli_args
}
