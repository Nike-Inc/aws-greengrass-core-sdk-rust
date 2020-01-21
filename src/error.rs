include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::convert::From;
use std::convert::Into;
use std::error::Error;
use std::ffi;
use std::fmt;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use crossbeam_channel::{SendError, RecvError};
use crate::handler::LambdaContext;

#[derive(Debug)]
pub enum GGError {
    OutOfMemory,
    InvalidParameter,
    InvalidState,
    InternalFailure,
    Terminate,
    NulError(ffi::NulError),
    InvalidString(ffi::IntoStringError),
    Unknown,
    HandlerChannelSendError(SendError<LambdaContext>),
    HandlerChannelRecvError(RecvError),
}

impl GGError {
    /// Returns the green grass error as a result.
    /// Success code will be Ok(())
    pub fn from_code(err_code: gg_error) -> Result<(), GGError> {
        match err_code {
            gg_error_GGE_SUCCESS => Ok(()),
            gg_error_GGE_OUT_OF_MEMORY => Err(Self::OutOfMemory),
            gg_error_GGE_INVALID_PARAMETER => Err(Self::InvalidParameter),
            gg_error_GGE_INVALID_STATE => Err(Self::InvalidState),
            gg_error_GGE_INTERNAL_FAILURE => Err(Self::InternalFailure),
            gg_error_GGE_TERMINATE => Err(Self::Terminate),
            _ => Err(Self::Unknown),
        }
    }
}

impl fmt::Display for GGError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => write!(f, "Process out of memory"),
            Self::InvalidParameter => write!(f, "Invalid input Parameter"),
            Self::InvalidState => write!(f, "Invalid State"),
            Self::InternalFailure => write!(f, "Internal Failure"),
            Self::Terminate => write!(f, "Remote signal to terminate received"),
            Self::NulError(ref e) => write!(f, "{}", e),
            Self::InvalidString(ref e) => write!(f, "{}", e),
            Self::HandlerChannelSendError(ref e) => write!(f, "Error sending to handler channel: {}", e),
            Self::HandlerChannelRecvError(ref e) => write!(f, "Error receving from handler channel: {}", e),
            _ => write!(f, "Unknown Error Occurred"),
        }
    }
}

impl Error for GGError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { 
        match self {
            Self::NulError(ref e) => Some(e),
            Self::InvalidString(ref e) => Some(e),
            Self::HandlerChannelSendError(ref e) => Some(e),
            Self::HandlerChannelRecvError(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<ffi::NulError> for GGError {
    fn from(e: ffi::NulError) -> Self {
        GGError::NulError(e)
    }
}

impl From<ffi::IntoStringError> for GGError {
    fn from(e: ffi::IntoStringError) -> Self {
        GGError::InvalidString(e)
    }
}

impl From<SendError<LambdaContext>> for GGError {
    fn from(e: SendError<LambdaContext>) -> Self {
        GGError::HandlerChannelSendError(e)
    }
}

impl From<RecvError> for GGError {
    fn from(e: RecvError) -> Self {
        GGError::HandlerChannelRecvError(e)
    }
}

impl Into<IOError> for GGError {
    fn into(self) -> IOError {
        IOError::new(IOErrorKind::Other, self)
    }
}