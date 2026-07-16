#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Ok = 0,
    DuplicatesFound = 1,
    Error = 2,
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        std::process::ExitCode::from(code as u8)
    }
}

/// Exit code contract for the TypeScript wrapper and CI scripts.
pub fn decide(fail_on_duplicates: bool, duplicates_found: bool) -> ExitCode {
    if fail_on_duplicates && duplicates_found {
        ExitCode::DuplicatesFound
    } else {
        ExitCode::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicates_is_ok() {
        assert_eq!(decide(true, false), ExitCode::Ok);
        assert_eq!(decide(false, false), ExitCode::Ok);
    }

    #[test]
    fn duplicates_only_fail_when_configured() {
        assert_eq!(decide(true, true), ExitCode::DuplicatesFound);
        assert_eq!(decide(false, true), ExitCode::Ok);
    }
}
