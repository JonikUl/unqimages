mod cli;
mod exit;
mod output;

use std::process::Command;
use std::{fs, io, time::Instant};
use unqimages_core::{find_duplicates, find_duplicates_with_staged, Config};

// Invariant: every error path returns ExitCode::Error (2); only a configured
// fail-on-duplicates finding can return ExitCode::DuplicatesFound (1).

use cli::{CliArgs, OutputFormat};
use exit::{decide, ExitCode};
use output::CliOutput;

fn main() -> std::process::ExitCode {
    let args = match CliArgs::parse_and_validate() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::Error.into();
        }
    };

    if let Some(cwd) = &args.cwd {
        if let Err(e) = std::env::set_current_dir(cwd) {
            eprintln!("error: failed to set working directory: {e}");
            return ExitCode::Error.into();
        }
    }

    let config = match load_config(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("error: failed to load config: {e}");
            return ExitCode::Error.into();
        }
    };

    let start = Instant::now();
    let result = if args.staged {
        match run_staged(&args, &config) {
            Ok(Some(result)) => result,
            Ok(None) => return ExitCode::Ok.into(),
            Err(e) => {
                eprintln!("error: scan failed: {e}");
                return ExitCode::Error.into();
            }
        }
    } else {
        match find_duplicates(&config) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("error: scan failed: {e}");
                return ExitCode::Error.into();
            }
        }
    };
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let output = CliOutput {
        duplicates: result.groups,
        scanned: result.scanned,
        elapsed_ms,
        used_cache: result.used_cache,
    };

    let mut stdout = io::stdout().lock();
    if let Err(e) = match args.output {
        OutputFormat::Json => output::print_json(&output, &mut stdout),
        OutputFormat::Table => output::print_table(&output, &mut stdout),
    } {
        eprintln!("error: failed to write output: {e}");
        return ExitCode::Error.into();
    }

    decide(config.fail_on_duplicates, !output.duplicates.is_empty()).into()
}

fn run_staged(args: &CliArgs, config: &Config) -> io::Result<Option<unqimages_core::ScanResult>> {
    let staged_paths = if args.paths.is_empty() {
        read_staged_paths_from_git()?
    } else {
        args.paths.clone()
    };

    if staged_paths.is_empty() {
        eprintln!("no staged image files to check");
        return Ok(None);
    }

    find_duplicates_with_staged(config, &staged_paths).map(Some)
}

fn read_staged_paths_from_git() -> io::Result<Vec<std::path::PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .stderr(std::process::Stdio::null())
        .output()?;

    if !output.status.success() {
        eprintln!("no git repository found or unable to read staged files");
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<std::path::PathBuf> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(std::path::PathBuf::from)
        .collect();

    Ok(paths)
}

fn load_config(args: &CliArgs) -> io::Result<Config> {
    let mut config = match &args.config {
        Some(path) => {
            let contents = fs::read_to_string(path)?;
            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid config file: {e}"),
                )
            })?
        }
        None => Config::default(),
    };

    if args.no_cache {
        config.ignore_cache = true;
    }

    Ok(config)
}
