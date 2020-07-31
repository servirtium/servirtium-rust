use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidMarkdownFormat,
    InvalidInteractionNumber,
    InvalidStatusCode,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMarkdownFormat => write!(f, "Markdown format is invalid"),
            Error::IoError(e) => write!(f, "IoError: {}", e),
            Error::InvalidStatusCode => write!(f, "The status code is invalid"),
            Error::InvalidInteractionNumber => write!(
                f,
                "Couldn't parse interaction number from the markdown file"
            ),
        }
    }
}
