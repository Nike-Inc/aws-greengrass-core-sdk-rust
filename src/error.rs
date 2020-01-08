include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GGError {
    OutOfMemory,
    InvalidParameter,
    InvalidState,
    InternalFailure,
    Terminate,
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
        // placeholder
        write!(f, "GGError: UNKNOWN")
    }
}

impl Error for GGError {}
