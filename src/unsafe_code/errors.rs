use std::error::Error;
use std::fmt;
use std::io;
use serde_json;
use bincode;


#[derive(Debug)]
pub enum UnsafeErrorKind {
    OpenEncoder(i32),
    OpenDecoder(i32),

    SendFrame(i32),
    ReceiveFrame(i32),

    SendPacket(i32),
    ReceivePacket(i32),

    OpenInput(i32),

    OpenSWSContext,
    SWSError,

    IOError(io::Error),
    ImageMagickError(&'static str),

    AVIOError(i32),
    WriteHeaderError(i32),
    WriteTrailerError(i32),
    WriteVideoFrameError(i32),

    BincodeError(Box<bincode::ErrorKind>),
    SerdeJsonError(serde_json::Error),
}

impl fmt::Display for UnsafeErrorKind {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &UnsafeErrorKind::OpenDecoder(ref i)          => write!(fmter, "An issue occured while opening the decoder: ERR {}",                            i),
            &UnsafeErrorKind::OpenEncoder(ref i)          => write!(fmter, "An issue occured while opening the encoder: ERR {}",                            i),
            &UnsafeErrorKind::ReceiveFrame(ref i)         => write!(fmter, "An issue occured while receiving a frame: ERR {}",                              i),
            &UnsafeErrorKind::ReceivePacket(ref i)        => write!(fmter, "An issue occured while receiving a packet: ERR {}",                             i),
            &UnsafeErrorKind::SendFrame(ref i)            => write!(fmter, "An issue occured while sending a frame: ERR {}",                                i),
            &UnsafeErrorKind::SendPacket(ref i)           => write!(fmter, "An issue occured while sending a packet: ERR {}",                               i),
            &UnsafeErrorKind::OpenInput(ref i)            => write!(fmter, "An issue occured while opening the input: ERR {}",                              i),
            &UnsafeErrorKind::OpenSWSContext              => write!(fmter, "An issue occured setting up SWS"),
            &UnsafeErrorKind::SWSError                    => write!(fmter, "An unknown error occured from SWS. Check the server logs for SWS entries"),
            &UnsafeErrorKind::ImageMagickError(ref e)     => write!(fmter, "{}",                                                                            e),
            &UnsafeErrorKind::IOError(ref e)              => e.fmt(fmter),
            &UnsafeErrorKind::AVIOError(ref e)            => write!(fmter, "An issue occured while trying to open the AVIO file: ERR {}",                   e),
            &UnsafeErrorKind::WriteHeaderError(ref e)     => write!(fmter, "An issue occured while trying to write the header of the AVIO file: ERR {}",    e),
            &UnsafeErrorKind::WriteTrailerError(ref e)         => write!(fmter, "An issue occured while trying to write the trailer of the AVIO file: ERR {}",   e),
            &UnsafeErrorKind::WriteVideoFrameError(ref e) => write!(fmter, "An issue occured while trying to write a video frame to the AVIO file: ERR {}", e),
            &UnsafeErrorKind::BincodeError(ref e)         => write!(fmter, "{:?}", e),
            &UnsafeErrorKind::SerdeJsonError(ref e)       => write!(fmter, "A Serde Error occured: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct UnsafeError {
    kind: UnsafeErrorKind,
}

impl UnsafeError {
    pub fn new(err_type: UnsafeErrorKind) -> UnsafeError {
        UnsafeError { kind: err_type }
    }
}

impl fmt::Display for UnsafeError {
    fn fmt(&self, fmter: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(fmter)
    }
}

impl Error for UnsafeError {
    fn description(&self) -> &str {
        "UnsafeError"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<io::Error> for UnsafeError {
    fn from(err: io::Error) -> UnsafeError {
        UnsafeError::new(UnsafeErrorKind::IOError(err))
    }
}

impl From<Box<bincode::ErrorKind>> for UnsafeError {
    fn from(err: Box<bincode::ErrorKind>) -> UnsafeError {
        UnsafeError::new(UnsafeErrorKind::BincodeError(err))
    }
}

impl From<serde_json::Error> for UnsafeError {
    fn from(err: serde_json::Error) -> UnsafeError {
        UnsafeError::new(UnsafeErrorKind::SerdeJsonError(err))
    }
}