use std::default::Default;
use std::convert::TryFrom;
use std::ffi::c_void;
use crate::error::GGError;
use crate::bindings::*;

/// The size of buffer we will use when reading results
/// from the C API
const BUFFER_SIZE: usize = 512;

/// Greengrass SDK request status enum
/// Maps to gg_request_status
#[derive(Debug, Clone)]
pub enum GGRequestStatus {
    /// function call returns expected payload type
    Success,
    /// function call is successfull, however lambda responded with an error
    Handled,
    /// function call is unsuccessfull, lambda exits abnormally
    Unhandled,
    /// System encounters unknown error. Check logs for more details
    Unknown,
    /// function call is throttled, try again
    Again,
}

impl TryFrom<&gg_request_status> for GGRequestStatus {
    type Error = GGError;

    fn try_from(value: &gg_request_status) -> Result<Self, Self::Error> {
        match value {
            &gg_request_status_GG_REQUEST_SUCCESS => Ok(Self::Success),
            &gg_request_status_GG_REQUEST_HANDLED => Ok(Self::Handled),
            &gg_request_status_GG_REQUEST_UNHANDLED => Ok(Self::Unhandled),
            &gg_request_status_GG_REQUEST_UNKNOWN => Ok(Self::Unknown),
            &gg_request_status_GG_REQUEST_AGAIN => Ok(Self::Again),
            _ => Err(Self::Error::Unknown(format!(
                "Unknown error code: {}",
                value
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GGRequestResponse {
    pub request_status: GGRequestStatus,
}

impl Default for GGRequestResponse {
    fn default() -> Self {
        GGRequestResponse {
            request_status: GGRequestStatus::Success,
        }
    }
}

impl TryFrom<&gg_request_result> for GGRequestResponse {
    type Error = GGError;

    fn try_from(value: &gg_request_result) -> Result<Self, Self::Error> {
        let status = GGRequestStatus::try_from(&value.request_status)?;
        Ok(GGRequestResponse {
            request_status: status,
        })
    }
}


/// Reads the response data from the gg_request_reqd call
pub(crate) fn read_response_data(req_to_read: gg_request) -> Result<Vec<u8>, GGError> {
    let mut bytes: Vec<u8> = Vec::new();

    unsafe {
        loop {
            let mut buffer = [0u8; BUFFER_SIZE];
            let mut read: usize = 0;
            let raw_read = &mut read as *mut usize;

            let read_res = gg_request_read(
                req_to_read,
                buffer.as_mut_ptr() as *mut c_void,
                BUFFER_SIZE,
                raw_read,
            );
            GGError::from_code(read_res)?;

            if read > 0 {
                bytes.extend_from_slice(&buffer[..read]);
            } else {
                break;
            }
        }
    }

    Ok(bytes)
}
