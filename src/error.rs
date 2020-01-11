include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::error::Error;
use std::fmt;
use std::ffi;
use std::convert::From;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::convert::Into;

#[derive(Debug)]
pub enum GGError {
    OutOfMemory,
    InvalidParameter,
    InvalidState,
    InternalFailure,
    Terminate,
    NulError(ffi::NulError),
    Unknown,
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
            _ => write!(f, "Unknown Error Occurred"),
        }
    }
}

impl Error for GGError {}

impl From<ffi::NulError> for GGError {
    fn from(e: ffi::NulError) -> Self {
        GGError::NulError(e)
    }
}

impl Into<IOError> for GGError {
    fn into(self) -> IOError {
        IOError::new(IOErrorKind::Other, self)
    }
}
