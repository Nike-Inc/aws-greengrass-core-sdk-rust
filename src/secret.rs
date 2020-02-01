include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use crate::GGResult;
use serde::Deserialize;
use std::convert::{AsRef, From};
use std::default::Default;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;

const BUFFER_SIZE: usize = 512;

/// Handles requests to the SecretManager secrets
/// that have been exposed to the green grass lambda
/// 
/// ```rust
/// use aws_greengrass_core_rust::secret::Secret;
/// 
/// let secret_result = Secret::for_secret_id("mysecret")
///     .with_secret_version(Some("version here".to_owned()))
///     .request();
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Secret {
    #[serde(rename = "ARN")]
    pub arn: String,
    pub name: String,
    pub version_id: String,
    pub secret_binary: Option<Vec<u8>>,
    pub secret_string: Option<String>,
    pub version_stages: Vec<String>,
    pub created_date: i64,
}

impl Secret {
    /// Creates a new SecretRequestBuilder using the specified secret_id
    ///
    /// * `secret_id` - The full arn or simple name of the secret
    pub fn for_secret_id(secret_id: &str) -> SecretRequestBuilder {
        SecretRequestBuilder::new(secret_id.to_owned())
    }

    /// For testing purposes.
    /// Can be called with default() to provide a string value
    pub fn with_secret_string(self, secret_string: Option<String>) -> Self {
        Secret {
            secret_string,
            ..self
        }
    }
}

/// Used to construct a request to send to acquire a secret from Greengrass
pub struct SecretRequestBuilder {
    pub secret_id: String,
    pub secret_version: Option<String>,
    pub secret_version_stage: Option<String>,
}

impl SecretRequestBuilder {
    /// The full id or simple name of the secret
    fn new(secret_id: String) -> Self {
        SecretRequestBuilder {
            secret_id,
            secret_version: None,
            secret_version_stage: None,
        }
    }

    /// Optional Secret version
    pub fn with_secret_version(self, secret_version: Option<String>) -> Self {
        SecretRequestBuilder {
            secret_version,
            ..self
        }
    }

    /// Optional secret stage
    pub fn with_secret_version_stage(self, secret_version_stage: Option<String>) -> Self {
        SecretRequestBuilder {
            secret_version_stage,
            ..self
        }
    }

    /// Executes the request and returns the secret
    pub fn request(&self) -> GGResult<Option<Secret>> {
        let response = read_secret(self)?;
        self.parse_response(&response)
    }

    fn parse_response(&self, response: &[u8]) -> GGResult<Option<Secret>> {
        match serde_json::from_slice::<Secret>(response) {
            Ok(secret) => Ok(Some(secret)),
            Err(e) => {
                // If parsing failed, see if we can parse it as an error response
                match serde_json::from_slice::<ErrorResponse>(response) {
                    Ok(er) => match er.status {
                        404 => Ok(None),
                        401 => Err(GGError::Unauthorized(format!(
                            "Not Authorized to access secret key: {}",
                            self.secret_id
                        ))),
                        _ => Err(GGError::Unknown(format!(
                            "status: {} - message: {}",
                            er.status, er.message
                        ))),
                    },
                    Err(_) => {
                        // Json parsing failed for another response, return wrapped version of the original error
                        Err(GGError::from(e))
                    }
                }
            }
        }
    }
}

/// Used for parsing Error responses from the secret call
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ErrorResponse {
    status: u16,
    message: String,
}

/// Reads the response data from the secret call
fn read_response_data(req_to_read: gg_request) -> Result<Vec<u8>, GGError> {
    let mut secret_bytes: Vec<u8> = Vec::new();

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
                secret_bytes.extend_from_slice(&buffer[..read]);
            } else {
                break;
            }
        }
    }

    Ok(secret_bytes)
}

/// Fetch the specified secrete from the green grass secret store
fn read_secret(builder: &SecretRequestBuilder) -> GGResult<Vec<u8>> {
    unsafe {
        let mut req: gg_request = ptr::null_mut();
        let req_init = gg_request_init(&mut req);
        GGError::from_code(req_init)?;

        let secret_name_c = CString::new(builder.secret_id.as_str()).map_err(GGError::from)?;
        let maybe_secret_version_c = if let Some(secret_version) = &builder.secret_version {
            Some(CString::new(secret_version.as_str()).map_err(GGError::from)?)
        } else {
            None
        };

        let maybe_secret_stage_c = if let Some(stage) = &builder.secret_version_stage {
            Some(CString::new(stage.as_str()).map_err(GGError::from)?)
        } else {
            None
        };

        let mut res = gg_request_result {
            request_status: gg_request_status_GG_REQUEST_SUCCESS,
        };

        let fetch_res = gg_get_secret_value(
            req,
            secret_name_c.as_ptr(),
            maybe_secret_version_c
                .map(|c| c.as_ptr())
                .unwrap_or(ptr::null() as *const c_char),
            maybe_secret_stage_c
                .map(|c| c.as_ptr())
                .unwrap_or(ptr::null() as *const c_char),
            &mut res,
        );

        GGError::from_code(fetch_res)?;

        let read_res = read_response_data(req);

        let close_res = gg_request_close(req);
        GGError::from_code(close_res)?;

        read_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARN: &'static str = "arn:aws:secretsmanager:us-west-2:701603852992:secret:greengrass-vendor-adapter-tls-secret-EZB0nM";
    const VERSION_ID: &'static str = "55acd8c0-ff58-4197-9b69-8772ea761ed4";
    const SECRET_STRING: &'static str = "foo";
    const NAME: &'static str = "greengrass-vendor-adapter-tls-secret";
    const CREATION_DATE: i64 = 1580414897159;
    const VERSION_STAGE: &'static str = "AWSCURRENT";

    fn version_stages() -> Vec<String> {
        vec![VERSION_STAGE.to_owned()]
    }

    fn test_response() -> String {
        format!("{{\"ARN\":\"{}\",\"Name\":\"{}\",\"VersionId\":\"{}\",\"SecretBinary\":null,\"SecretString\":\"{}\",\"VersionStages\":{:?},\"CreatedDate\":{} }}", ARN, NAME, VERSION_ID, SECRET_STRING, version_stages(), CREATION_DATE)
    }

    #[test]
    fn test_parse_response() {
        let response = test_response();
        println!("{}", response);
        let secret: Secret = serde_json::from_str(&response).unwrap();
        assert_eq!(ARN, secret.arn);
        assert_eq!(VERSION_ID, secret.version_id);
        assert_eq!(SECRET_STRING, secret.secret_string.unwrap());
        assert_eq!(NAME, secret.name);
        assert_eq!(CREATION_DATE, secret.created_date);
        assert_eq!(version_stages(), secret.version_stages);
    }
}
