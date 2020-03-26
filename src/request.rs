//! Provides utilities for working with handling green grass response objects.
//! Generally consumers of the API will not use items in this module, except in error cases.
//! In error cases the GGRequestResponse struct will be embedded in the the GGError::ErrorResponse.
//! In this case we are exposing it as I we don't know every possible error that AWS returns.
//!
//! # Examples
//!
//! ## Error Handling Example
//! ```rust
//! use aws_greengrass_core_rust::iotdata::IOTDataClient;
//! use aws_greengrass_core_rust::error::GGError;
//! use aws_greengrass_core_rust::request::GGRequestStatus;
//! match IOTDataClient::default().publish("my topic", "my payload") {
//!     Ok(_) => println!("Yay, it worked!"),
//!     Err(GGError::ErrorResponse(resp)) => {
//!         match resp.request_status {
//!             GGRequestStatus::Again => eprintln!("You should retry again because you were throttled"),
//!             _ => eprintln!("An error that is probably unrecoverable happened."),
//!         }
//!     }
//!     _ => eprintln!("Another greengrass system error occurred"),
//! }
//! ```
use crate::bindings::*;
use crate::error::GGError;
use crate::GGResult;
use log::error;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::default::Default;
use std::ffi::c_void;

/// The size of buffer we will use when reading results
/// from the C API
const BUFFER_SIZE: usize = 512;

/// Greengrass SDK request status enum
/// Maps to gg_request_status
#[derive(Debug, Clone, PartialEq, Serialize)]
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

impl TryFrom<gg_request_status> for GGRequestStatus {
    type Error = GGError;

    #[allow(non_upper_case_globals)]
    fn try_from(value: gg_request_status) -> Result<Self, Self::Error> {
        match value {
            gg_request_status_GG_REQUEST_SUCCESS => Ok(Self::Success),
            gg_request_status_GG_REQUEST_HANDLED => Ok(Self::Handled),
            gg_request_status_GG_REQUEST_UNHANDLED => Ok(Self::Unhandled),
            gg_request_status_GG_REQUEST_UNKNOWN => Ok(Self::Unknown),
            gg_request_status_GG_REQUEST_AGAIN => Ok(Self::Again),
            _ => Err(Self::Error::Unknown(format!(
                "Unknown error code: {}",
                value
            ))),
        }
    }
}

/// Represents the response returned by greengrass via the gg_read_response C function.
/// Generally this will only be used internally to the API (Success), except in the case of most error responses.
/// In most server side error responses (where we receive a json error object) this object will be contained
/// inside the GGError::ErrorResponse error.
#[derive(Debug, Clone, Serialize)]
pub struct GGRequestResponse {
    /// The status of the GG request
    pub request_status: GGRequestStatus,
    /// If there was an error response, what was it.
    pub error_response: Option<ErrorResponse>,
}

impl GGRequestResponse {
    pub fn with_error_response(self, error_response: Option<ErrorResponse>) -> Self {
        GGRequestResponse {
            error_response,
            ..self
        }
    }

    /// Returns true if the request status is anything other than GGRequestStatus::Success
    pub fn is_error(&self) -> bool {
        self.request_status != GGRequestStatus::Success
    }

    /// Ok(()) if there is no error, otherwise the error we found
    /// This is useful for requests that do not contain a body
    pub(crate) fn to_error_result(&self, req: gg_request) -> GGResult<()> {
        match self.determine_error(req) {
            ErrorState::Error(e) => Err(e),
            _ => Ok(()), // Ignore the NotFoundError too
        }
    }

    /// Attempt to read the response body.
    /// If the response is an error the error will be returned else the body in bytes.
    /// This is useful for requests that contain a body
    pub(crate) fn read(&self, req: gg_request) -> GGResult<Option<Vec<u8>>> {
        match self.determine_error(req) {
            ErrorState::None => {
                let data = read_response_data(req)?;
                Ok(Some(data))
            }
            ErrorState::NotFoundError => Ok(None),
            ErrorState::Error(e) => Err(e),
        }
    }

    /// If the response is an error, return it as Some(GGError)
    /// None if it isn't an error
    fn determine_error(&self, req: gg_request) -> ErrorState {
        if self.is_error() {
            // If this is an error than try to read the response body
            // So we can see what kind of error it is
            let read_result =
                read_response_data(req).and_then(|e| ErrorResponse::try_from(e.as_slice()));
            match read_result {
                Ok(err_resp) => match err_resp.code {
                    404 => ErrorState::NotFoundError,
                    401 => ErrorState::Error(GGError::Unauthorized(err_resp.message)),
                    _ => ErrorState::Error(GGError::ErrorResponse(self.clone())),
                },
                Err(e) => {
                    error!(
                        "Error trying to read error response for response: {:?} error: {}",
                        self, e
                    );
                    ErrorState::Error(e)
                }
            }
        } else {
            ErrorState::None
        }
    }
}

/// There are three states instead of two (why I didn't use option).
/// This is because I want to capture the 404 error response and wrap it as Option in an attempt
/// to make the API feel more idiomatic
enum ErrorState {
    /// All other errors
    Error(GGError),
    /// A 404 error was returned
    NotFoundError,
    /// No Error response was found
    None,
}

impl Default for GGRequestResponse {
    fn default() -> Self {
        GGRequestResponse {
            request_status: GGRequestStatus::Success,
            error_response: None,
        }
    }
}

impl TryFrom<&gg_request_result> for GGRequestResponse {
    type Error = GGError;

    fn try_from(value: &gg_request_result) -> Result<Self, Self::Error> {
        let status = GGRequestStatus::try_from(value.request_status)?;
        Ok(GGRequestResponse {
            request_status: status,
            error_response: None,
        })
    }
}

/// Represents an Error Response from greengrass JSON object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    /// AWS uses the HTTP status codes even though this is over MQTT
    pub code: u16,
    /// Message related to the error
    pub message: String,
    /// The time this error occurred
    pub timestamp: u64,
}

impl TryFrom<&[u8]> for ErrorResponse {
    type Error = GGError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value).map_err(Self::Error::from)
    }
}

/// Reads the response data from the gg_request_reqd call
fn read_response_data(req_to_read: gg_request) -> Result<Vec<u8>, GGError> {
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

#[macro_export]
macro_rules! with_request {
    ($req:expr, $expr:block) => {{
        let req_init = gg_request_init(&mut $req);
        GGError::from_code(req_init)?;
        let wrapper = move || {
            //this will catch the return
            $expr
        };
        let output = wrapper();
        let close_res = gg_request_close($req);
        GGError::from_code(close_res)?;
        output
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ptr;

    const READ_DATA: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Malesuada fames ac turpis egestas maecenas pharetra. Ornare massa eget egestas purus viverra accumsan in nisl nisi. Dolor morbi non arcu risus. Vehicula ipsum a arcu cursus vitae. Luctus accumsan tortor posuere ac ut consequat semper viverra. At tempor commodo ullamcorper a lacus vestibulum sed. Dui ut ornare lectus sit amet. Tristique magna sit amet purus gravida quis blandit turpis. Duis at consectetur lorem donec. Amet cursus sit amet dictum sit. Lacus viverra vitae congue eu consequat ac felis donec et.

Suscipit tellus mauris a diam. Odio tempor orci dapibus ultrices in. Ullamcorper velit sed ullamcorper morbi tincidunt ornare massa eget egestas. Suspendisse interdum consectetur libero id faucibus nisl tincidunt eget. Non diam phasellus vestibulum lorem sed risus. Amet justo donec enim diam vulputate ut pharetra sit amet. Est ullamcorper eget nulla facilisi etiam. Ut etiam sit amet nisl purus in mollis nunc. Vel eros donec ac odio tempor. Nascetur ridiculus mus mauris vitae ultricies.

Purus sit amet volutpat consequat mauris nunc. Odio ut enim blandit volutpat. Etiam tempor orci eu lobortis elementum. Praesent tristique magna sit amet purus gravida. Interdum velit laoreet id donec ultrices tincidunt arcu non sodales. Sed pulvinar proin gravida hendrerit lectus. Lacus laoreet non curabitur gravida arcu. Turpis cursus in hac habitasse platea dictumst quisque. Tempor orci eu lobortis elementum nibh. Pellentesque massa placerat duis ultricies lacus sed turpis. Ut porttitor leo a diam sollicitudin tempor id eu nisl. Justo laoreet sit amet cursus. Ultrices in iaculis nunc sed augue lacus viverra vitae. Nec tincidunt praesent semper feugiat nibh sed pulvinar proin. Sem nulla pharetra diam sit amet nisl. Suspendisse potenti nullam ac tortor vitae purus faucibus. Odio morbi quis commodo odio aenean. Justo nec ultrices dui sapien eget mi.

Tincidunt nunc pulvinar sapien et ligula ullamcorper malesuada. Orci sagittis eu volutpat odio facilisis mauris sit amet. Nunc non blandit massa enim. Dui ut ornare lectus sit amet est placerat in egestas. Risus sed vulputate odio ut enim blandit volutpat. Pellentesque adipiscing commodo elit at imperdiet dui accumsan. Dolor magna eget est lorem ipsum. Velit sed ullamcorper morbi tincidunt ornare massa eget. Amet commodo nulla facilisi nullam vehicula ipsum. Velit dignissim sodales ut eu sem. Sed id semper risus in hendrerit gravida rutrum. Sit amet porttitor eget dolor morbi. Blandit turpis cursus in hac. Scelerisque felis imperdiet proin fermentum leo.

Facilisi nullam vehicula ipsum a arcu cursus vitae congue. Massa massa ultricies mi quis hendrerit. Sit amet facilisis magna etiam. Duis convallis convallis tellus id interdum velit laoreet id donec. Neque laoreet suspendisse interdum consectetur libero. Sed vulputate odio ut enim blandit volutpat maecenas volutpat blandit. Amet volutpat consequat mauris nunc. Erat nam at lectus urna duis convallis convallis tellus. Consectetur a erat nam at lectus urna. Iaculis at erat pellentesque adipiscing commodo elit at imperdiet. Volutpat blandit aliquam etiam erat. Semper quis lectus nulla at volutpat. Orci a scelerisque purus semper eget. Fermentum et sollicitudin ac orci phasellus egestas tellus rutrum.

Ultrices mi tempus imperdiet nulla. Purus in massa tempor nec feugiat nisl pretium fusce id. Praesent tristique magna sit amet purus. Facilisis volutpat est velit egestas dui. Sed egestas egestas fringilla phasellus faucibus scelerisque. Convallis a cras semper auctor. Viverra accumsan in nisl nisi. Aliquet nec ullamcorper sit amet risus. Massa sed elementum tempus egestas sed sed risus pretium. Tortor consequat id porta nibh. In tellus integer feugiat scelerisque varius morbi enim nunc. Adipiscing commodo elit at imperdiet dui accumsan. Tincidunt id aliquet risus feugiat in ante metus. Interdum varius sit amet mattis. Sit amet massa vitae tortor condimentum. Purus non enim praesent elementum. Vestibulum sed arcu non odio euismod lacinia at quis risus. Tempor id eu nisl nunc mi ipsum faucibus vitae aliquet.

Vestibulum rhoncus est pellentesque elit. Feugiat nisl pretium fusce id velit ut tortor pretium. Tortor consequat id porta nibh venenatis cras sed felis eget. Velit scelerisque in dictum non consectetur a. Hendrerit gravida rutrum quisque non. Porta non pulvinar neque laoreet suspendisse interdum consectetur. Tellus rutrum tellus pellentesque eu tincidunt. Sed arcu non odio euismod lacinia at quis. Netus et malesuada fames ac turpis egestas integer eget. Vitae justo eget magna fermentum iaculis eu non. Tincidunt praesent semper feugiat nibh sed pulvinar proin. Sodales ut eu sem integer vitae justo. Enim blandit volutpat maecenas volutpat blandit aliquam. Non odio euismod lacinia at quis risus sed. Mollis nunc sed id semper. Non enim praesent elementum facilisis leo vel fringilla. Viverra vitae congue eu consequat ac felis donec. Placerat orci nulla pellentesque dignissim. Libero nunc consequat interdum varius sit amet mattis vulputate enim. Sed turpis tincidunt id aliquet.

Porttitor massa id neque aliquam vestibulum morbi blandit cursus. Lacus suspendisse faucibus interdum posuere lorem. Mauris cursus mattis molestie a iaculis at erat. Blandit massa enim nec dui nunc. Arcu bibendum at varius vel pharetra vel turpis nunc eget. Tristique et egestas quis ipsum suspendisse ultrices gravida dictum. Adipiscing tristique risus nec feugiat in. Egestas sed sed risus pretium quam. At ultrices mi tempus imperdiet nulla malesuada pellentesque elit eget. Ac placerat vestibulum lectus mauris ultrices. Id volutpat lacus laoreet non curabitur gravida arcu ac tortor. Etiam erat velit scelerisque in dictum. At erat pellentesque adipiscing commodo. Mollis aliquam ut porttitor leo a diam sollicitudin tempor. Habitant morbi tristique senectus et netus et malesuada fames. Cras sed felis eget velit aliquet sagittis id consectetur purus. Ut sem nulla pharetra diam sit amet nisl suscipit adipiscing. Risus nec feugiat in fermentum posuere urna nec.

Nibh mauris cursus mattis molestie a iaculis. Diam ut venenatis tellus in. Nisl tincidunt eget nullam non nisi est sit. Adipiscing commodo elit at imperdiet dui accumsan sit amet. Tristique senectus et netus et. Vulputate ut pharetra sit amet aliquam. Non consectetur a erat nam at lectus urna duis. Aliquet sagittis id consectetur purus ut faucibus. Eget felis eget nunc lobortis mattis aliquam faucibus. Nisl tincidunt eget nullam non nisi. Nunc eget lorem dolor sed viverra ipsum nunc. Suspendisse faucibus interdum posuere lorem. Nisl condimentum id venenatis a condimentum. Malesuada pellentesque elit eget gravida cum sociis. Porta lorem mollis aliquam ut porttitor leo a diam. Maecenas volutpat blandit aliquam etiam.

Parturient montes nascetur ridiculus mus mauris vitae ultricies. Suspendisse sed nisi lacus sed viverra. Adipiscing elit pellentesque habitant morbi tristique senectus et netus et. Gravida in fermentum et sollicitudin. Sem et tortor consequat id porta nibh venenatis. Volutpat commodo sed egestas egestas fringilla phasellus faucibus scelerisque. Amet cursus sit amet dictum sit amet justo donec enim. Nulla facilisi cras fermentum odio eu feugiat pretium nibh ipsum. Fermentum leo vel orci porta non pulvinar neque laoreet. Nunc sed id semper risus in hendrerit. Aliquet sagittis id consectetur purus ut faucibus pulvinar elementum. Tincidunt vitae semper quis lectus nulla at volutpat. Vel facilisis volutpat est velit egestas dui id ornare arcu. Vivamus arcu felis bibendum ut tristique et egestas quis. Sed vulputate odio ut enim blandit volutpat. Vel pharetra vel turpis nunc. Orci dapibus ultrices in iaculis nunc sed augue lacus. Vitae tempus quam pellentesque nec nam aliquam sem et tortor. Eget lorem dolor sed viverra ipsum. Sapien pellentesque habitant morbi tristique senectus et netus et malesuada.";

    #[test]
    fn test_read_response_data() {
        GG_REQUEST_READ_BUFFER.with(|buffer| buffer.replace(READ_DATA.to_owned()));
        let mut req: gg_request = ptr::null_mut();
        let result = gg_request_init(&mut req);
        assert_eq!(result, gg_error_GGE_SUCCESS);

        let result = read_response_data(req).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result, READ_DATA);
    }

    #[test]
    fn test_try_from_gg_request_status() {
        assert_eq!(GGRequestStatus::try_from(gg_request_status_GG_REQUEST_SUCCESS).unwrap(), GGRequestStatus::Success);
        assert_eq!(GGRequestStatus::try_from(gg_request_status_GG_REQUEST_HANDLED).unwrap(), GGRequestStatus::Handled);
        assert_eq!(GGRequestStatus::try_from(gg_request_status_GG_REQUEST_UNHANDLED).unwrap(), GGRequestStatus::Unhandled);
        assert_eq!(GGRequestStatus::try_from(gg_request_status_GG_REQUEST_UNKNOWN).unwrap(), GGRequestStatus::Unknown);
        assert_eq!(GGRequestStatus::try_from(gg_request_status_GG_REQUEST_AGAIN).unwrap(), GGRequestStatus::Again);
        assert!(GGRequestStatus::try_from(9999).is_err());
    }

}
