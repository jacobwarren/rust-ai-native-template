//! Replaceable pure-domain example: validated input and explicit errors.
use std::fmt;
pub mod board;

/// A nonempty, bounded, single-line display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong,
    ControlCharacter,
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "name must not be empty",
            Self::TooLong => "name must be at most 80 Unicode scalar values",
            Self::ControlCharacter => "name must not contain control characters",
        })
    }
}

impl std::error::Error for NameError {}

impl TryFrom<&str> for Name {
    type Error = NameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        // Reject controls before trimming so newlines are never silently accepted.
        if raw.chars().any(char::is_control) {
            return Err(NameError::ControlCharacter);
        }
        let name = raw.trim();
        if name.is_empty() {
            return Err(NameError::Empty);
        }
        if name.chars().count() > 80 {
            return Err(NameError::TooLong);
        }
        Ok(Self(name.to_owned()))
    }
}

/// Deterministic business behavior; no process, storage, or network access.
pub fn greet(name: &Name) -> String {
    format!("Hello, {}!", name.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_boundaries() {
        assert_eq!(Name::try_from("  "), Err(NameError::Empty));
        assert_eq!(Name::try_from("a\nb"), Err(NameError::ControlCharacter));
        assert!(Name::try_from("é".repeat(80).as_str()).is_ok());
        assert_eq!(
            Name::try_from("é".repeat(81).as_str()),
            Err(NameError::TooLong)
        );
    }

    #[test]
    fn preserves_unicode_and_normalizes_surrounding_spaces() {
        let name = Name::try_from("  Zoë  ").expect("valid name");
        assert_eq!(greet(&name), "Hello, Zoë!");
    }
}
