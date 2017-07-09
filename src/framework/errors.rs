use std::error::Error;
use std::fmt;
use std::convert::From;

#[derive(Debug, PartialEq, Hash)]
pub enum FrameworkErrorKind {
    OpenUpstream,
    OpenDownstream,

    CloseUpstream,
    CloseDownstream,
}

impl fmt::Display for FrameworkErrorKind {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &FrameworkErrorKind::OpenUpstream => write!(fmter, "Couldn't properly open an upstream source."),
            &FrameworkErrorKind::OpenDownstream => write!(fmter, "Couldn't properly open a downstream sink."),
            &FrameworkErrorKind::CloseUpstream => write!(fmter, "Couldn't properly close an upstream source."),
            &FrameworkErrorKind::CloseDownstream => write!(fmter, "Couldn't properly close a downstream sink."),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct FrameworkError(FrameworkErrorKind);

impl FrameworkError {
    fn new(kind: FrameworkErrorKind) -> FrameworkError {
        FrameworkError(kind)
    }
}

impl From<FrameworkErrorKind> for FrameworkError {
    fn from(kind: FrameworkErrorKind) -> FrameworkError {
        FrameworkError::new(kind)
    }
}

impl fmt::Display for FrameworkError {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmter)
    }
}

impl Error for FrameworkError {
    fn description(&self) -> &str {
        "FrameworkError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}