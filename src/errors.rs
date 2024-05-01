use crate::protocol::ParseMessageError;
use std::fmt::Display;

pub enum Errors {
    ParseMessage(ParseMessageError),
    Serde(serde_json::Error),
    Other(&'static str),
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseMessage(e) => write!(f, "{}", e),
            Self::Serde(e) => write!(f, "{}", e),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl From<ParseMessageError> for Errors {
    fn from(value: ParseMessageError) -> Self {
        Self::ParseMessage(value)
    }
}

impl From<serde_json::Error> for Errors {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<&'static str> for Errors {
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }
}
