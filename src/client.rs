include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::convert::TryFrom;
use std::default::Default;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use log::{info, warn};

#[cfg(all(test, feature = "mock"))]
use self::mock::*;

use crate::error::GGError;
use crate::GGResult;

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

#[derive(Clone)]
pub struct IOTDataClient {
    /// When the mock feature is turned on this field will contain captured input
    /// and values to be returned
    #[cfg(all(test, feature = "mock"))]
    pub mocks: MockHolder,
}

#[cfg(not(all(test, feature = "mock")))]
impl IOTDataClient {
    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(&self, topic: &str, message: T) -> GGResult<GGRequestResponse> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len();
        self.publish_raw(topic, as_bytes, size)
    }

    /// Raw publish method that wraps gg_request_init, gg_publish
    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<GGRequestResponse> {
        unsafe {
            info!("Publishing message of length {} to topic {}", read, topic);
            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;

            let topic_c = CString::new(topic).map_err(GGError::from)?;
            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };

            let pub_res = gg_publish(
                req,
                topic_c.as_ptr(),
                buffer as *const _ as *const c_void,
                read,
                &mut res,
            );
            GGError::from_code(pub_res)?;

            let close_res = gg_request_close(req);
            GGError::from_code(close_res)?;

            GGRequestResponse::try_from(&res)
        }
    }

}

#[cfg(all(test, feature = "mock"))]
impl IOTDataClient {
    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(&self, topic: &str, message: T) -> GGResult<GGRequestResponse> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len();
        self.publish_raw(topic, as_bytes, size)
    }

    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<GGRequestResponse> {
        warn!("Mock publish_raw is being executed!!! This should not happen in prod!!!!");
        self.mocks.publish_raw_inputs.borrow_mut().push(PublishRawInput(topic.to_owned(), buffer.to_owned(), read));
        // If there is an output return the output
        if let Some(output) = self.mocks.publish_raw_outputs.borrow_mut().pop() {
            output
        }
        else {
            Ok(GGRequestResponse::default())
        }
    }

    /// When the mock feature is turned on this will contain captured inputs and return
    /// provided outputs
    #[cfg(all(test, feature = "mock"))]
    pub fn with_mocks(self, mocks: MockHolder) -> Self {
        IOTDataClient {
            mocks,
            ..self
        }
    }

}

impl Default for IOTDataClient {
    fn default() -> Self {
        IOTDataClient {
            #[cfg(all(test, feature = "mock"))]
            mocks: MockHolder::default(),
        }
    }
}

#[cfg(all(test, feature = "mock"))]
pub mod mock {
    use super::*;
    use std::cell::RefCell;

    /// Represents the capture input from the publish_raw field
    #[derive(Debug, Clone)]
    pub struct PublishRawInput(pub String, pub Vec<u8>, pub usize);

    /// Use to override input and output when the mock feature is enabled
    #[derive(Debug)]
    pub struct MockHolder {
        pub publish_raw_inputs: RefCell<Vec<PublishRawInput>>,
        pub publish_raw_outputs: RefCell<Vec<GGResult<GGRequestResponse>>>,
    }

    impl MockHolder {
        pub fn with_publish_raw_outputs(self, publish_raw_outputs: Vec<GGResult<GGRequestResponse>>) -> Self {
            MockHolder {
                publish_raw_outputs: RefCell::new(publish_raw_outputs),
                ..self
            }
        }
    }

    impl Default for MockHolder {
        fn default() -> Self {
            MockHolder {
                publish_raw_inputs: RefCell::new(vec![]),
                publish_raw_outputs: RefCell::new(vec![]),
            }
        }
    }

    // Clone is needed because the contract of IOTDataClient has clone
    // We don't necessary clone every field
    impl Clone for MockHolder {
        fn clone(&self) -> Self {
            MockHolder {
                publish_raw_inputs: RefCell::new(self.publish_raw_inputs.borrow().clone()),
                // NOTE: We can't copy the outputs since result isn't cloneable, so just empty it
                publish_raw_outputs: RefCell::new(vec![]),
            }
        }
    }

    unsafe impl Send for MockHolder {}
    unsafe impl Sync for MockHolder {}

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_publish_str() {
            let topic = "foo";
            let message = "this is my message";

            let mocks =
                MockHolder::default().with_publish_raw_outputs(vec![Ok(GGRequestResponse::default())]);
            let client = IOTDataClient::default().with_mocks(mocks);
            let response = client.publish(topic, message).unwrap();
            println!("response: {:?}", response);

            let PublishRawInput(raw_topic, raw_bytes, raw_read) = &client.mocks.publish_raw_inputs.borrow()[0];
            assert_eq!(raw_topic, topic);
        }
    }
}