use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io;
use std::str;
use std::net::AddrParseError;
use std::sync::mpsc::RecvError;
use std::env;

use config::server_configuration::ServerConfigurationError;

use unsafe_code::UnsafeError;

use rusqlite;

#[derive(Debug)]
pub enum ServerErrorKind {
    IOError(io::Error),
    SQLiteError(rusqlite::Error),
    IronError,
    UTF8Error(str::Utf8Error),
    AddrParseErr(AddrParseError),
    UnsafeError(UnsafeError),
    RecvError(RecvError),
    EnvVarError(env::VarError),
    ServerConfigError(ServerConfigurationError),

}

impl fmt::Display for ServerErrorKind {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ServerErrorKind::IOError(ref err) => err.fmt(fmter),
            &ServerErrorKind::SQLiteError(ref err) => err.fmt(fmter),
            &ServerErrorKind::IronError => write!(fmter, "An Iron error occured"),
            &ServerErrorKind::UTF8Error(ref err) => err.fmt(fmter),
            &ServerErrorKind::AddrParseErr(ref err) => err.fmt(fmter),
            &ServerErrorKind::UnsafeError(ref err) => err.fmt(fmter),
            &ServerErrorKind::RecvError(ref err) => err.fmt(fmter),
            &ServerErrorKind::EnvVarError(ref err) => err.fmt(fmter),
            &ServerErrorKind::ServerConfigError(ref err) => err.fmt(fmter),
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

impl From<UnsafeError> for ServerError {
    fn from(err: UnsafeError) -> ServerError {
        ServerError::new(ServerErrorKind::UnsafeError(err))
    }
}

impl From<RecvError> for ServerError {
    fn from(err: RecvError) -> ServerError {
        ServerError::new(ServerErrorKind::RecvError(err))
    }
}

impl From<ServerConfigurationError> for ServerError {
    fn from(err: ServerConfigurationError) -> ServerError {
        ServerError::new(ServerErrorKind::ServerConfigError(err))
    }
}

impl From<env::VarError> for ServerError {
    fn from(err: env::VarError) -> ServerError {
        ServerError::new(ServerErrorKind::EnvVarError(err))
    }
}