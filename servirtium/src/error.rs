use hyper::http;
use std::{fmt::Display, io, sync};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    PoisonedLock,
    InvalidDomainName,
    InvalidStatusCode,
    NotConfigured,
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidBody,
    HyperError(hyper::Error),
    ParseUriError,
    HttpError(http::Error),
    InteractionManagerError(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IoError: {}", e),
            Error::PoisonedLock => write!(f, "The lock was poisoned"),
            Error::InvalidStatusCode => write!(f, "The status code is invalid"),
            Error::NotConfigured => write!(f, "The server hasn't been configured"),
            Error::InvalidHeaderName => write!(f, "Invalid header name"),
            Error::InvalidHeaderValue => write!(f, "Invalid header value"),
            Error::InvalidBody => write!(f, "Invalid body"),
            Error::HyperError(e) => write!(f, "Hyper error: {}", e),
            Error::ParseUriError => write!(f, "Parse URI Error"),
            Error::HttpError(e) => write!(f, "Http Error: {}", e),
            Error::InvalidDomainName => write!(f, "Couldn't parse the domain name"),
            Error::InteractionManagerError(e) => write!(f, "Markdown manager error: {}", e),
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
