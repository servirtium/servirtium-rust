use std::fmt::Formatter;
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidMarkdownFormat,
    InvalidInteractionNumber,
    InvalidStatusCode,
    MarkdownsDiffer(MarkdownsDifferenceType, MarkdownsDifferenceLocation),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMarkdownFormat => write!(f, "Markdown format is invalid"),
            Error::Io(e) => write!(f, "IoError: {}", e),
            Error::InvalidStatusCode => write!(f, "The status code is invalid"),
            Error::InvalidInteractionNumber => write!(
                f,
                "Couldn't parse interaction number from the markdown file"
            ),
            Error::MarkdownsDiffer(difference_type, location) => {
                write!(f, "{} - {}", location, difference_type)
            }
        }
    }
}

#[derive(Debug)]
pub enum MarkdownsDifferenceType {
    Body(MarkdownsBodyDifference),
    Header(MarkdownsHeaderDifference),
}

impl Display for MarkdownsDifferenceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownsDifferenceType::Body(b) => write!(f, "{}", b),
            MarkdownsDifferenceType::Header(h) => write!(f, "{}", h),
        }
    }
}

#[derive(Debug)]
pub enum MarkdownsDifferenceLocation {
    Request,
    Response,
}

impl Display for MarkdownsDifferenceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownsDifferenceLocation::Request => write!(f, "Request"),
            MarkdownsDifferenceLocation::Response => write!(f, "Response"),
        }
    }
}

#[derive(Debug)]
pub struct MarkdownsBodyDifference {
    pub line: u32,
    pub column: u32,
    pub old_context: String,
    pub new_context: String,
}

impl Display for MarkdownsBodyDifference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bodies differ at line {}, column {}. Old: \"{}\". New: \"{}\"",
            self.line,
            self.column,
            self.old_context.escape_default(),
            self.new_context.escape_default()
        )
    }
}

#[derive(Debug)]
pub struct MarkdownsHeaderDifference {
    pub header_name: String,
    pub old_header_value: Option<String>,
    pub new_header_value: Option<String>,
}

impl Display for MarkdownsHeaderDifference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let old_header = if let Some(existing_header_value) = self.old_header_value.as_ref() {
            format!("\"{}\": \"{}\"", self.header_name, existing_header_value)
        } else {
            "<no header value>".into()
        };

        let new_header = if let Some(new_header_value) = self.new_header_value.as_ref() {
            format!("\"{}\": \"{}\"", self.header_name, new_header_value)
        } else {
            "<no header value>".into()
        };

        write!(
            f,
            "Headers differ. old - {}, new - {}",
            old_header, new_header
        )
    }
}
