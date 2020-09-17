use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum NormalizeError {
    UrlParseError,
    InternalError,
    UrlEncodeError,
    RegexParseError(String),
}

impl Display for NormalizeError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            NormalizeError::UrlParseError => write!(f, "Url parse error."),
            NormalizeError::InternalError => write!(f, "Internal error."),
            NormalizeError::UrlEncodeError => write!(f, "Url encode error."),
            NormalizeError::RegexParseError(regex) => write!(f, "Regex parse error:{}", regex),
        }
    }
}

impl Error for NormalizeError {
    fn description(&self) -> &str {
        "Url canonicalization failed:"
    }
}