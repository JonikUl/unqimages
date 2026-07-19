use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "unqimages-core")]
#[command(about = "Find duplicate images in a project")]
pub struct CliArgs {
    /// JSON config path.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Directory to scan.
    #[arg(long, value_name = "PATH")]
    pub cwd: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub output: OutputFormat,

    /// Ignore the cache and recompute all hashes.
    #[arg(long)]
    pub no_cache: bool,

    /// Check staged images as new files.
    #[arg(long)]
    pub staged: bool,

    /// Staged file paths passed by lint-staged or another caller.
    #[arg(trailing_var_arg = true)]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

impl CliArgs {
    /// Validate at the boundary so the rest of the binary sees only typed args.
    pub fn parse_and_validate() -> Result<Self, String> {
        let args = Self::parse();
        args.validate()
    }

    fn validate(self) -> Result<Self, String> {
        if let Some(config_path) = &self.config {
            if !config_path.exists() {
                return Err(format!(
                    "config file does not exist: {}",
                    config_path.display()
                ));
            }
        }

        if let Some(cwd) = &self.cwd {
            if !cwd.exists() {
                return Err(format!(
                    "working directory does not exist: {}",
                    cwd.display()
                ));
            }
            if !cwd.is_dir() {
                return Err(format!(
                    "working directory is not a directory: {}",
                    cwd.display()
                ));
            }
        }

        if !self.staged && !self.paths.is_empty() {
            return Err(
                "unexpected file paths: use --staged when passing staged file paths".to_string(),
            );
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_args() {
        let args = CliArgs::try_parse_from(["unqimages-core"]).unwrap();
        assert!(args.config.is_none());
        assert!(args.cwd.is_none());
        assert_eq!(args.output, OutputFormat::Json);
    }

    #[test]
    fn parse_all_args() {
        let args = CliArgs::try_parse_from([
            "unqimages-core",
            "--config",
            "config.json",
            "--cwd",
            "/tmp",
            "--output",
            "table",
        ])
        .unwrap();
        assert_eq!(args.config, Some(PathBuf::from("config.json")));
        assert_eq!(args.cwd, Some(PathBuf::from("/tmp")));
        assert_eq!(args.output, OutputFormat::Table);
    }

    #[test]
    fn invalid_output_format_rejected() {
        let result = CliArgs::try_parse_from(["unqimages-core", "--output", "yaml"]);
        assert!(result.is_err());
    }

    #[test]
    fn missing_config_file_rejected() {
        let args = CliArgs::try_parse_from([
            "unqimages-core",
            "--config",
            "/definitely/missing/config.json",
        ])
        .unwrap();
        // clap parses successfully; validation catches the missing file.
        assert!(args.validate().is_err());
    }

    #[test]
    fn missing_cwd_rejected() {
        let args = CliArgs::try_parse_from(["unqimages-core", "--cwd", "/definitely/missing/dir"])
            .unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn file_as_cwd_rejected() {
        let args = CliArgs::try_parse_from(["unqimages-core", "--cwd", "/etc/hosts"]).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn parse_staged_flag() {
        let args = CliArgs::try_parse_from(["unqimages-core", "--staged"]).unwrap();
        assert!(args.staged);
        assert!(args.paths.is_empty());
    }

    #[test]
    fn parse_staged_with_paths() {
        let args =
            CliArgs::try_parse_from(["unqimages-core", "--staged", "a.png", "b.jpg"]).unwrap();
        assert!(args.staged);
        assert_eq!(
            args.paths,
            vec![PathBuf::from("a.png"), PathBuf::from("b.jpg")]
        );
    }

    #[test]
    fn paths_without_staged_rejected() {
        let args = CliArgs::try_parse_from(["unqimages-core", "a.png"]).unwrap();
        assert!(args.validate().is_err());
    }
}
