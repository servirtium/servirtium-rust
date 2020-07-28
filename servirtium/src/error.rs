use hyper::http;
use std::{fmt::Display, io, sync};

#[derive(Debug)]
pub enum Error {
    InvalidMarkdownFormat,
    InvalidMarkdownPath,
    IoError(io::Error),
    PoisonedLock,
    InvalidStatusCode,
    NotConfigured,
    ReqwestError(reqwest::Error),
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidBody,
    HyperError(hyper::Error),
    ParseUriError,
    HttpError(http::Error),
    UnknownError,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMarkdownFormat => write!(f, "The markdown format was poisoned"),
            Error::IoError(e) => write!(f, "IoError: {}", e),
            Error::PoisonedLock => write!(f, "The lock was poisoned"),
            Error::InvalidStatusCode => write!(f, "The status code is invalid"),
            Error::NotConfigured => write!(f, "The server hasn't been configured"),
            Error::ReqwestError(e) => write!(f, "reqwest error: {}", e),
            Error::InvalidHeaderName => write!(f, "Invalid header name"),
            Error::InvalidHeaderValue => write!(f, "Invalid header value"),
            Error::InvalidBody => write!(f, "Invalid body"),
            Error::HyperError(e) => write!(f, "Hyper error: {}", e),
            Error::ParseUriError => write!(f, "Parse URI Error"),
            Error::UnknownError => write!(f, "Unknown Servirtium Error"),
            Error::HttpError(e) => write!(f, "Http Error: {}", e),
            Error::InvalidMarkdownPath => {
                write!(f, "The markdown path should point to a markdown file")
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(_: sync::PoisonError<T>) -> Self {
        Error::PoisonedLock
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ReqwestError(e)
    }
}

impl From<hyper::header::InvalidHeaderName> for Error {
    fn from(_: hyper::header::InvalidHeaderName) -> Self {
        Error::InvalidHeaderName
    }
}

impl From<hyper::header::InvalidHeaderValue> for Error {
    fn from(_: hyper::header::InvalidHeaderValue) -> Self {
        Error::InvalidHeaderValue
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::HyperError(e)
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::HttpError(e)
    }
}
