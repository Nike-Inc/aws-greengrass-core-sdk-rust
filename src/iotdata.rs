/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

//! Provides the ability to publish MQTT topics
use log::info;
use serde::ser::Serialize;
use std::convert::{TryInto, TryFrom};
use std::default::Default;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

#[cfg(all(test, feature = "mock"))]
use self::mock::*;

use crate::bindings::*;
use crate::error::GGError;
use crate::request::GGRequestResponse;
use crate::with_request;
use crate::GGResult;

/// What actions should be taken if an MQTT queue is full
#[derive(Clone, Debug)]
pub enum QueueFullPolicy {
    /// GGC will deliver messages to as many targets as possible
    BestEffort,
    /// GGC will either deliver messages to all targets and return request
    /// successful status or deliver to no targets and return a
    /// GGError::ErrorResponse with a GGRequestResponse with a status of 'Again'
    AllOrError,
}

impl QueueFullPolicy {
    fn to_queue_full_c(&self) -> gg_queue_full_policy_options {
        match self {
            Self::BestEffort => gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_BEST_EFFORT,
            Self::AllOrError => gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_ALL_OR_ERROR,
        }
    }
}

/// Options that can be supplied when the client publishes
#[derive(Clone, Debug)]
pub struct PublishOptions {
    pub queue_full_policy: QueueFullPolicy,
}

impl PublishOptions {
    /// Define a custom policy when publishing from this client
    pub fn with_queue_full_policy(self, queue_full_policy: QueueFullPolicy) -> Self {
        PublishOptions { queue_full_policy }
    }
}

impl Default for PublishOptions {
    fn default() -> Self {
        PublishOptions {
            queue_full_policy: QueueFullPolicy::BestEffort,
        }
    }
}

/// Provides MQTT publishing to Greengrass lambda functions
///
/// # Examples
///
/// ## Basic Publishing
/// ```rust
/// use aws_greengrass_core_rust::iotdata::IOTDataClient;
/// let client = IOTDataClient::default();
/// if let Err(e) = client.publish("some_topic", r#"{"msg": "some payload"}"#) {
///     eprintln!("An error occurred publishing: {}", e);
/// }
/// ```
///
/// ## Publishing with Options
/// ```rust
/// use aws_greengrass_core_rust::iotdata::{PublishOptions, QueueFullPolicy, IOTDataClient};
/// let options = PublishOptions::default().with_queue_full_policy(QueueFullPolicy::AllOrError);
/// let client = IOTDataClient::default().with_publish_options(Some(options));
/// if let Err(e) = client.publish("some_topic", r#"{"msg": "some payload"}"#) {
///     eprintln!("An error occurred publishing: {}", e);
/// }
/// ```
#[derive(Clone)]
pub struct IOTDataClient {
    /// The policy that this client will use when publishing
    /// if one has been defined
    pub publish_options: Option<PublishOptions>,
    /// When the mock feature is turned on this field will contain captured input
    /// and values to be returned
    #[cfg(all(test, feature = "mock"))]
    pub mocks: MockHolder,
}

impl IOTDataClient {
    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(&self, topic: &str, message: T) -> GGResult<()> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len().try_into().map_err(GGError::from)?;
        self.publish_raw(topic, as_bytes, size)
    }

    /// Publish anything that is a deserializable serde object
    pub fn publish_json<T: Serialize>(&self, topic: &str, message: T) -> GGResult<()> {
        let bytes = serde_json::to_vec(&message).map_err(GGError::from)?;
        self.publish(topic, &bytes)
    }

    /// Raw publish method that wraps gg_request_init, gg_publish
    #[cfg(not(all(test, feature = "mock")))]
    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: size_t) -> GGResult<()> {
        self.publish_with_options(topic, buffer, read)
    }

    /// This wraps publish_internal and will set any publish options if publish options were specified
    /// The primary reason this is a separate function from publish_internal is to ensure that if
    /// options is specified we clean up the pointer we create on error
    fn publish_with_options(&self, topic: &str, buffer: &[u8], read: size_t) -> GGResult<()> {
        unsafe {
            // If options were defined, initialize the options pointer and
            // set queue policy
            let options_c: Option<gg_publish_options> = if let Some(po) = &self.publish_options {
                let mut opts_c: gg_publish_options = ptr::null_mut();
                let init_resp = gg_publish_options_init(&mut opts_c);
                GGError::from_code(init_resp)?;

                let queue_policy_c = po.queue_full_policy.to_queue_full_c();
                let policy_resp = gg_publish_options_set_queue_full_policy(opts_c, queue_policy_c);
                if let Err(e) = GGError::from_code(policy_resp) {
                    // make sure that we free the options pointer
                    let free_resp = gg_publish_options_free(opts_c);
                    GGError::from_code(free_resp)?;
                    return Err(e);
                }

                Some(opts_c)
            } else {
                None
            };

            let publish_result = self.publish_internal(topic, buffer, read, options_c);

            // Clean up the options pointer if we created one
            if let Some(opts) = options_c {
                let free_resp = gg_publish_options_free(opts);
                GGError::from_code(free_resp)?;
            }
            publish_result
        }
    }

    /// Raw publish method that wraps gg_request_init, gg_publish
    unsafe fn publish_internal(
        &self,
        topic: &str,
        buffer: &[u8],
        read: size_t,
        options_tuple: Option<gg_publish_options>,
    ) -> GGResult<()> {
        info!("Publishing message of length {} to topic {}", read, topic);
        let topic_c = CString::new(topic).map_err(GGError::from)?;
        let mut req: gg_request = ptr::null_mut();
        with_request!(req, {
            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };
            let pub_res = if let Some(options_c) = options_tuple {
                gg_publish_with_options(
                    req,
                    topic_c.as_ptr(),
                    buffer as *const _ as *const c_void,
                    read,
                    options_c,
                    &mut res,
                )
            } else {
                gg_publish(
                    req,
                    topic_c.as_ptr(),
                    buffer as *const _ as *const c_void,
                    read,
                    &mut res,
                )
            };
            GGError::from_code(pub_res)?;
            GGRequestResponse::try_from(&res)?.to_error_result(req)
        })
    }

    /// Optionally define a publishing options for this Client
    #[allow(clippy::needless_update)]
    pub fn with_publish_options(self, publish_options: Option<PublishOptions>) -> Self {
        IOTDataClient {
            publish_options,
            ..self
        }
    }

    // -----------------------------------
    // Mock methods
    // -----------------------------------

    #[cfg(all(test, feature = "mock"))]
    pub fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<()> {
        log::warn!("Mock publish_raw is being executed!!! This should not happen in prod!!!!");
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
            publish_options: None,
            #[cfg(all(test, feature = "mock"))]
            mocks: MockHolder::default(),
        }
    }
}

/// Provides mock testing utilities
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
    use serde_json::Value;

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_publish_raw() {
        reset_test_state();
        let topic = "my_topic";
        let my_payload = b"This is my payload.";
        let my_payload_len = my_payload.len().try_into().unwrap();
        IOTDataClient::default()
            .publish_raw(topic, my_payload, my_payload_len)
            .unwrap();
        GG_PUBLISH_ARGS.with(|ref_cell| {
            let args = ref_cell.borrow();
            assert_eq!(args.topic, topic);
            assert_eq!(args.payload, my_payload);
            assert_eq!(args.payload_size, my_payload_len);
        });
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_publish_with_options() {
        reset_test_state();
        let topic = "another topic";
        let my_payload: Value = serde_json::from_str(r#"{"foo": "bar"}"#).unwrap();
        let publish_options =
            PublishOptions::default().with_queue_full_policy(QueueFullPolicy::AllOrError);
        let client = IOTDataClient::default().with_publish_options(Some(publish_options));
        client.publish_json(topic, my_payload.clone()).unwrap();

        GG_PUBLISH_WITH_OPTIONS_ARGS.with(|rc| {
            let args = rc.borrow();
            let my_payload_as_bytes = serde_json::to_vec(&my_payload).unwrap();
            let my_payload_len: size_t = my_payload_as_bytes.len().try_into().unwrap();
            assert_eq!(args.topic, topic);
            assert_eq!(args.payload, my_payload_as_bytes);
            assert_eq!(args.payload_size, my_payload_len);
        });
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
        GG_PUBLISH_OPTION_INIT_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_PUBLISH_OPTION_FREE_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_PUBLISH_OPTIONS_SET_QUEUE_FULL_POLICY.with(|rc| {
            assert_eq!(
                *rc.borrow(),
                gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_ALL_OR_ERROR
            )
        });
    }
}
