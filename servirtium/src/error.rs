use hyper::http;
use std::{fmt::Display, io, sync};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    PoisonedLock,
    InvalidDomainName,
    InvalidStatusCode,
    NotConfigured,
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidBody,
    Hyper(hyper::Error),
    ParseUri,
    Http(http::Error),
    InteractionManager(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IoError: {}", e),
            Error::PoisonedLock => write!(f, "The lock was poisoned"),
            Error::InvalidStatusCode => write!(f, "The status code is invalid"),
            Error::NotConfigured => write!(f, "The server hasn't been configured"),
            Error::InvalidHeaderName => write!(f, "Invalid header name"),
            Error::InvalidHeaderValue => write!(f, "Invalid header value"),
            Error::InvalidBody => write!(f, "Invalid body"),
            Error::Hyper(e) => write!(f, "Hyper error: {}", e),
            Error::ParseUri => write!(f, "Parse URI Error"),
            Error::Http(e) => write!(f, "Http Error: {}", e),
            Error::InvalidDomainName => write!(f, "Couldn't parse the domain name"),
            Error::InteractionManager(e) => write!(f, "Markdown manager error: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
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
        Error::Hyper(e)
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::Http(e)
    }
}
