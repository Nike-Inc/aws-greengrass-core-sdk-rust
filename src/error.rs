/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

//! Provides error handling

use crate::bindings::*;
use crate::handler::LambdaContext;
use crate::request::GGRequestResponse;
use crossbeam_channel::{RecvError, SendError};
use log::error;
use serde_json::Error as SerdeError;
use std::convert::From;
use std::convert::Into;
use std::error::Error;
use std::ffi;
use std::fmt;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::string::FromUtf8Error;

/// Provices a wrapper for the various errors that are incurred both working with the
/// GreenGrass C SDK directly or from the content of the results from it's responses (e.g. http status codes in json response objects)
#[derive(Debug)]
pub enum GGError {
    /// Maps to the C API GGE_OUT_OF_MEMORY response
    OutOfMemory,
    /// Maps to the C API GGE_INVALID_PARAMETER response
    InvalidParameter,
    /// Maps to the C API GGE_INVALID_STATE response
    InvalidState,
    /// Maps to the C API GGE_INTERNAL_FAILURE response
    InternalFailure,
    /// Maps to the C API GGE_TERMINATE response
    Terminate,
    /// If null pointer from the C API that cannot be recovered from is encountered
    NulError(ffi::NulError),
    /// C String cannot be coerced into a Rust String
    InvalidString(String),
    /// If receive an error type from the C API that isn't known
    Unknown(String),
    /// If there are issues in communicating to the Handler  
    HandlerChannelSendError(SendError<LambdaContext>),
    /// If there are issues in communicating to the Handler  
    HandlerChannelRecvError(RecvError),
    /// If an AWS response contains an unauthorized error code
    Unauthorized(String),
    /// Thrown if there is an error with the JSON content we received from AWS
    JsonError(SerdeError),
    /// When the green grass response is an error
    /// If the error is a 404, it should be handled as an Option instead. Otherwise
    /// this error type can be returned.
    ErrorResponse(GGRequestResponse),
}

impl GGError {
    /// Returns the green grass error as a result.
    /// Success code will be Ok(())
    #[allow(non_upper_case_globals)]
    pub fn from_code(err_code: gg_error) -> Result<(), GGError> {
        match err_code {
            gg_error_GGE_SUCCESS => Ok(()),
            gg_error_GGE_OUT_OF_MEMORY => Err(Self::OutOfMemory),
            gg_error_GGE_INVALID_PARAMETER => Err(Self::InvalidParameter),
            gg_error_GGE_INVALID_STATE => Err(Self::InvalidState),
            gg_error_GGE_INTERNAL_FAILURE => Err(Self::InternalFailure),
            gg_error_GGE_TERMINATE => Err(Self::Terminate),
            _ => {
                error!("Received unknown error code: {}", err_code);
                Err(Self::Unknown(format!("Unknown error code: {}", err_code)))
            }
        }
    }

    // Converts the error to an IoError
    #[allow(clippy::wrong_self_convention)]
    pub fn as_ioerror(self) -> IOError {
        IOError::new(IOErrorKind::Other, self)
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
            Self::HandlerChannelSendError(ref e) => {
                write!(f, "Error sending to handler channel: {}", e)
            }
            Self::HandlerChannelRecvError(ref e) => {
                write!(f, "Error receving from handler channel: {}", e)
            }
            Self::JsonError(ref e) => write!(f, "Error parsing response: {}", e),
            Self::Unknown(ref s) => write!(f, "{}", s),
            Self::InvalidString(ref e) => write!(f, "Invalid String: {}", e),
            Self::Unauthorized(ref s) => write!(f, "{}", s),
            Self::ErrorResponse(ref r) => write!(f, "Green responded with error: {:?}", r),
        }
    }
}

impl Error for GGError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NulError(ref e) => Some(e),
            Self::HandlerChannelSendError(ref e) => Some(e),
            Self::HandlerChannelRecvError(ref e) => Some(e),
            Self::JsonError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<ffi::NulError> for GGError {
    fn from(e: ffi::NulError) -> Self {
        GGError::NulError(e)
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
        self.as_ioerror()
    }
}

impl From<FromUtf8Error> for GGError {
    fn from(e: FromUtf8Error) -> Self {
        Self::InvalidString(format!("{}", e))
    }
}

impl From<SerdeError> for GGError {
    fn from(e: SerdeError) -> Self {
        Self::JsonError(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_from_code() {
        assert!(GGError::from_code(gg_error_GGE_SUCCESS).is_ok());

        match GGError::from_code(gg_error_GGE_INTERNAL_FAILURE) {
            Err(GGError::InternalFailure) => (),
            _ => panic!("Expected InternalFailure"),
        };

        match GGError::from_code(gg_error_GGE_INVALID_PARAMETER) {
            Err(GGError::InvalidParameter) => (),
            _ => panic!("Expected InvalidParameter"),
        };

        match GGError::from_code(gg_error_GGE_INVALID_STATE) {
            Err(GGError::InvalidState) => (),
            _ => panic!("Expected InvalidState"),
        };

        match GGError::from_code(gg_error_GGE_TERMINATE) {
            Err(GGError::Terminate) => (),
            _ => panic!("Expected Terminate"),
        };

        match GGError::from_code(gg_error_GGE_OUT_OF_MEMORY) {
            Err(GGError::OutOfMemory) => (),
            _ => panic!("Expected InternalFailure"),
        };

        match GGError::from_code(999) {
            Err(GGError::Unknown(_)) => (),
            _ => panic!("Expected InternalFailure"),
        };
    }

    #[test]
    fn test_serde_error() {
        let result: Result<Value, GGError> =
            serde_json::from_str("sdflkasdf {d92").map_err(GGError::from);
        assert!(result.is_err());
        let unwrapped = result.unwrap_err();
        assert!(unwrapped.source().is_some());
        assert!(format!("{}", unwrapped).len() > 10);
    }
}
