use crate::bindings::*;
use crate::error::GGError;
use crate::request::GGRequestResponse;
use crate::with_request;
use crate::GGResult;
use serde::Deserialize;
use std::convert::From;
use std::convert::TryFrom;
use std::default::Default;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

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
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
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
        if let Some(response) = read_secret(self)? {
            Ok(Some(self.parse_response(&response)?))
        } else {
            Ok(None)
        }
    }

    fn parse_response(&self, response: &[u8]) -> GGResult<Secret> {
        serde_json::from_slice::<Secret>(response).map_err(GGError::from)
    }
}

/// Fetch the specified secrete from the green grass secret store
fn read_secret(builder: &SecretRequestBuilder) -> GGResult<Option<Vec<u8>>> {
    unsafe {
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

        let mut req: gg_request = ptr::null_mut();
        with_request!(req, {
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
            let response = GGRequestResponse::try_from(&res)?;
            response.read(req)
        })
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

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_for_secret_id_request() {
        reset_test_state();
        let secret_id = "my_secret_id";
        GG_REQUEST_READ_BUFFER.with(|rc| rc.replace(test_response().into_bytes()));
        let secret = Secret::for_secret_id(secret_id).request().unwrap().unwrap();
        let assert_secret_string = test_response();
        assert_eq!(
            secret,
            serde_json::from_str::<Secret>(assert_secret_string.as_str()).unwrap()
        );
        GG_GET_SECRET_VALUE_ARGS.with(|rc| {
            let borrowed = rc.borrow();
            assert_eq!(borrowed.secret_id, secret_id);
        });
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_for_secret_gg_error() {
        let error = gg_error_GGE_INVALID_STATE;
        GG_GET_SECRET_VALUE_RETURN.with(|rc| rc.replace(error));
        let secret_id = "failed_secret_id";
        let secret = Secret::for_secret_id(secret_id).request();
        assert!(secret.is_err());
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
        if let Err(GGError::InvalidState) = secret {
            // don't fail
        } else {
            panic!("There should have been an Invalid Err");
        }
    }
}
