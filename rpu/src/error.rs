use crate::prelude::*;
use std::fmt;

/// Represents an RPU Error.
pub struct RPUError {
    pub message: String,
    pub line: usize,
    pub file: Option<String>,
}

impl RPUError {
    pub fn new<M>(message: M, line: usize) -> Self
    where
        M: Into<String>,
    {
        Self {
            message: message.into(),
            line,
            file: None,
        }
    }

    pub fn loc<M>(message: M, location: &Location) -> Self
    where
        M: Into<String>,
    {
        Self {
            message: message.into(),
            line: location.line,
            file: Some(location.file.clone()),
        }
    }
}

impl fmt::Display for RPUError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{} in {} at line {}.", self.message, file, self.line)
        } else {
            write!(f, "{} at line {}.", self.message, self.line)
        }
    }
}
