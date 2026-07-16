mod cli;
mod exit;
mod output;

use std::{fs, io, time::Instant};
use unqimages_core::{find_duplicates, Config};

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
    let result = match find_duplicates(&config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::Error.into();
        }
    };
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let output = CliOutput {
        duplicates: result.groups,
        scanned: result.scanned,
        elapsed_ms,
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

fn load_config(args: &CliArgs) -> io::Result<Config> {
    match &args.config {
        Some(path) => {
            let contents = fs::read_to_string(path)?;
            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid config file: {e}"),
                )
            })
        }
        None => Ok(Config::default()),
    }
}
