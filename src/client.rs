use log::{info, warn};
use serde::ser::Serialize;
use std::convert::TryFrom;
use std::default::Default;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

#[cfg(all(test, feature = "mock"))]
use self::mock::*;

use crate::bindings::*;
use crate::error::GGError;
use crate::request::{read_response_data, ErrorResponse, GGRequestResponse};
use crate::try_clean;
use crate::GGResult;

#[derive(Clone)]
pub struct IOTDataClient {
    /// When the mock feature is turned on this field will contain captured input
    /// and values to be returned
    #[cfg(all(test, feature = "mock"))]
    pub mocks: MockHolder,
}

impl IOTDataClient {
    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(&self, topic: &str, message: T) -> GGResult<()> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len();
        self.publish_raw(topic, as_bytes, size)
    }

    /// Publish anything that is a deserializable serde object
    pub fn publish_json<T: Serialize>(&self, topic: &str, message: T) -> GGResult<()> {
        let bytes = serde_json::to_vec(&message).map_err(GGError::from)?;
        self.publish(topic, &bytes)
    }

    /// Raw publish method that wraps gg_request_init, gg_publish
    #[cfg(not(all(test, feature = "mock")))]
    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<()> {
        unsafe {
            info!("Publishing message of length {} to topic {}", read, topic);
            let topic_c = CString::new(topic).map_err(GGError::from)?;

            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;

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
            try_clean!(req, GGError::from_code(pub_res));

            let response = try_clean!(req, GGRequestResponse::try_from(&res));
            if response.is_error() {
                let result = try_clean!(req, read_response_data(req));
                let error_response = try_clean!(req, ErrorResponse::try_from(result.as_slice()));

                let response_2 = response.with_error_response(Some(error_response));
                let close_res = gg_request_close(req);
                GGError::from_code(close_res)?;
                Err(GGError::ErrorResponse(response_2))
            } else {
                let close_res = gg_request_close(req);
                GGError::from_code(close_res)?;
                Ok(())
            }
        }
    }

    // -----------------------------------
    // Mock methods
    // -----------------------------------

    #[cfg(all(test, feature = "mock"))]
    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<()> {
        warn!("Mock publish_raw is being executed!!! This should not happen in prod!!!!");
        self.mocks
            .publish_raw_inputs
            .borrow_mut()
            .push(PublishRawInput(topic.to_owned(), buffer.to_owned(), read));
        // If there is an output return the output
        if let Some(output) = self.mocks.publish_raw_outputs.borrow_mut().pop() {
            output
        } else {
            Ok(())
        }
    }

    /// When the mock feature is turned on this will contain captured inputs and return
    /// provided outputs
    #[cfg(all(test, feature = "mock"))]
    pub fn with_mocks(self, mocks: MockHolder) -> Self {
        IOTDataClient { mocks, ..self }
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
        pub publish_raw_outputs: RefCell<Vec<GGResult<()>>>,
    }

    impl MockHolder {
        pub fn with_publish_raw_outputs(self, publish_raw_outputs: Vec<GGResult<()>>) -> Self {
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

    // Note: This is to get past compile issues.. Mock testing for threads
    // could result in undefined behavior
    unsafe impl Send for MockHolder {}
    unsafe impl Sync for MockHolder {}

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_publish_str() {
            let topic = "foo";
            let message = "this is my message";

            let mocks = MockHolder::default().with_publish_raw_outputs(vec![Ok(())]);
            let client = IOTDataClient::default().with_mocks(mocks);
            let response = client.publish(topic, message).unwrap();
            println!("response: {:?}", response);

            let PublishRawInput(raw_topic, raw_bytes, raw_read) =
                &client.mocks.publish_raw_inputs.borrow()[0];
            assert_eq!(raw_topic, topic);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_publish_raw() {
        let topic = "my_topic";
        let my_payload = b"This is my payload.";
        IOTDataClient::default().publish_raw(topic, my_payload, my_payload.len()).unwrap();
        GG_PUBLISH_ARGS.with(|ref_cell| {
            let args = ref_cell.borrow();
            assert_eq!(args.topic, topic);
            assert_eq!(args.payload, my_payload);
            assert_eq!(args.payload_size, my_payload.len());
        })
    }

}
