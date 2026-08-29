use std::fmt;

/// A single crate-wide error type. Every failure message is fully formed by the function
/// that raises it (mirrors the TS adapter's plain `Error` with a hand-built message) rather
/// than a structured variant per failure kind — callers match on message content in tests,
/// same as the TS `toThrowError(/regex/)` assertions this crate's tests port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeError(pub String);

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ThemeError {}
