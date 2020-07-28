use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    DateRangeNotSupported(u16, u16),
    NotRecognizedByClimateWeb,
    DeserializationError(quick_xml::DeError),
    ReqwestError(reqwest::Error),
    IoError(io::Error),
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ReqwestError(e)
    }
}

impl From<quick_xml::DeError> for Error {
    fn from(e: quick_xml::DeError) -> Self {
        Error::DeserializationError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DateRangeNotSupported(from_date, to_date) => {
                write!(f, "Date range {}-{} not supported", from_date, to_date)
            }
            Error::NotRecognizedByClimateWeb => write!(f, "Not recognized by ClimateWeb"),
            Error::ReqwestError(e) => write!(f, "{}", e.to_string()),
            Error::DeserializationError(e) => write!(f, "{}", e.to_string()),
            Error::IoError(e) => write!(f, "{}", e.to_string()),
        }
    }
}
