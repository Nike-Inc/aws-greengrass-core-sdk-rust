include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::convert::TryFrom;
use std::default::Default;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;

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
    pub inner: Arc<dyn IOTDataClientInner>,
}

impl IOTDataClient {
    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(&self, topic: &str, message: T) -> GGResult<GGRequestResponse> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len();
        self.inner.publish_raw(topic, as_bytes, size)
    }

    pub fn with_inner(self, inner: Arc<dyn IOTDataClientInner>) -> Self {
        IOTDataClient { inner }
    }
}

impl Default for IOTDataClient {
    fn default() -> Self {
        IOTDataClient {
            inner: Arc::new(DefaultIODataClientInner),
        }
    }
}

pub trait IOTDataClientInner {
    fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<GGRequestResponse>;
}

#[derive(Clone)]
struct DefaultIODataClientInner;

impl IOTDataClientInner for DefaultIODataClientInner {
    /// Raw publish method that wraps gg_request_init, gg_publish
    fn publish_raw(&self, topic: &str, buffer: &[u8], read: usize) -> GGResult<GGRequestResponse> {
        unsafe {
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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::test::CallHolder;
    use std::rc::Rc;

    /// Represents that parameters that were pushed to the IOTDataClientInner#publish_raw call
    pub struct PublishRaw(String, Vec<u8>, usize);

    /// Mock implementation of IOTDataClientInner
    pub struct MockInner {
        pub publish_raw_call: Rc<CallHolder<PublishRaw>>,
    }

    impl IOTDataClientInner for MockInner {
        fn publish_raw(
            &self,
            topic: &str,
            buffer: &[u8],
            read: usize,
        ) -> GGResult<GGRequestResponse> {
            self.publish_raw_call
                .push(PublishRaw(topic.to_owned(), buffer.to_owned(), read));
            Ok(GGRequestResponse::default())
        }
    }

    #[test]
    fn test_publish_str() {
        let topic = "foo";
        let message = "this is my message";

        let call_holder = Rc::new(CallHolder::<PublishRaw>::new());

        let inner = MockInner {
            publish_raw_call: Rc::clone(&call_holder),
        };

        let client = IOTDataClient::default().with_inner(Arc::new(inner));

        let response = client.publish(topic, message).unwrap();

        let PublishRaw(raw_topic, raw_bytes, raw_read) = &call_holder.calls()[0];
        assert_eq!(raw_topic, topic);
    }
}
