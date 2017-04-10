use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use std::str;
use std::net::AddrParseError;
use rusqlite;

#[derive(Debug)]
pub enum ServerErrorKind {
    IOError(io::Error),
    SQLiteError(rusqlite::Error),
    IronError,
    UTF8Error(str::Utf8Error),
    AddrParseErr(AddrParseError),
}

impl fmt::Display for ServerErrorKind {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ServerErrorKind::IOError(ref err) => err.fmt(fmter),
            &ServerErrorKind::SQLiteError(ref err) => err.fmt(fmter),
            &ServerErrorKind::IronError => write!(fmter, "An Iron error occured"),
            &ServerErrorKind::UTF8Error(ref err) => err.fmt(fmter),
            &ServerErrorKind::AddrParseErr(ref err) => err.fmt(fmter),
        }
    }
}

#[derive(Debug)]
pub struct ServerError {
    error_type: ServerErrorKind
}

impl ServerError {
    pub fn new(type_of_err: ServerErrorKind) -> ServerError {
        ServerError { error_type: type_of_err }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        self.error_type.fmt(fmter)
    }
}

impl Error for ServerError {
    fn description(&self) -> &str {
        "ServerError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> ServerError {
        ServerError::new(ServerErrorKind::IOError(err))
    }
}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> ServerError {
        ServerError::new(ServerErrorKind::SQLiteError(err))
    }
}

impl From<str::Utf8Error> for ServerError {
    fn from(err: str::Utf8Error) -> ServerError {
        ServerError::new(ServerErrorKind::UTF8Error(err))
    }
}

impl From<AddrParseError> for ServerError {
    fn from(err: AddrParseError) -> ServerError {
        ServerError::new(ServerErrorKind::AddrParseErr(err))
    }
}