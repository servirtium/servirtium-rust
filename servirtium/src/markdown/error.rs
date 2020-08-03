use std::{fmt::Display, io};

#[derive(Debug)]
pub struct MarkdownsBodyDifference {
    pub line: u32,
    pub column: u32,
    pub old_context: String,
    pub new_context: String,
}

#[derive(Debug)]
pub struct MarkdownsHeaderDifference {
    pub header_name: String,
    pub old_header_value: Option<String>,
    pub new_header_value: Option<String>,
}

#[derive(Debug)]
pub enum MarkdownsDifferenceType {
    Body(MarkdownsBodyDifference),
    Header(MarkdownsHeaderDifference),
}

#[derive(Debug)]
pub enum MarkdownsDifferenceLocation {
    Request,
    Response,
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidMarkdownFormat,
    InvalidInteractionNumber,
    InvalidStatusCode,
    MarkdownsDiffer(MarkdownsDifferenceType, MarkdownsDifferenceLocation),
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
            Error::MarkdownsDiffer(difference_type, location) => {
                let location_description = match location {
                    MarkdownsDifferenceLocation::Request => "Request",
                    MarkdownsDifferenceLocation::Response => "Response",
                };

                match difference_type {
                    MarkdownsDifferenceType::Body(MarkdownsBodyDifference {
                        line,
                        column,
                        old_context,
                        new_context,
                    }) => write!(
                        f,
                        "{} bodies differ at line {}, column {}. Old: \"{}\". New: \"{}\"",
                        location_description,
                        line,
                        column,
                        old_context.escape_default(),
                        new_context.escape_default()
                    ),
                    MarkdownsDifferenceType::Header(MarkdownsHeaderDifference {
                        old_header_value,
                        header_name,
                        new_header_value,
                    }) => {
                        let old_header = if let Some(existing_header_value) = old_header_value {
                            format!("\"{}\": \"{}\"", header_name, existing_header_value)
                        } else {
                            "<no header value>".into()
                        };

                        let new_header = if let Some(new_header_value) = new_header_value {
                            format!("\"{}\": \"{}\"", header_name, new_header_value)
                        } else {
                            "<no header value>".into()
                        };

                        write!(
                            f,
                            "{} headers differ. old - {}, new - {}",
                            location_description, old_header, new_header
                        )
                    }
                }
            }
        }
    }
}
