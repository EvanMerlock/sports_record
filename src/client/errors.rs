use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use std::net::AddrParseError;
use std::str;
use unsafe_code::UnsafeError;

#[derive(Debug)]
pub enum ClientErrorKind {
    UnsafeError(UnsafeError),
    IOError(io::Error),
    AddrParseErr(AddrParseError),
}

impl fmt::Display for ClientErrorKind {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ClientErrorKind::IOError(ref err) => err.fmt(fmter),
            &ClientErrorKind::AddrParseErr(ref err) => err.fmt(fmter),
            &ClientErrorKind::UnsafeError(ref err) => err.fmt(fmter),
        }
    }
}

#[derive(Debug)]
pub struct ClientError {
    error_type: ClientErrorKind,
}

impl ClientError {
    pub fn new(type_of_err: ClientErrorKind) -> ClientError {
        ClientError { error_type: type_of_err }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        self.error_type.fmt(fmter)
    }
}

impl Error for ClientError {
    fn description(&self) -> &str {
        "ClientError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> ClientError {
        ClientError::new(ClientErrorKind::IOError(err))
    }
}

impl From<AddrParseError> for ClientError {
    fn from(err: AddrParseError) -> ClientError {
        ClientError::new(ClientErrorKind::AddrParseErr(err))
    }
}

impl From<UnsafeError> for ClientError {
    fn from(err: UnsafeError) -> ClientError {
        ClientError::new(ClientErrorKind::UnsafeError(err))
    }
}