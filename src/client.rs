include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use log::info;
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

use crate::GGResult;
use crate::error::GGError;

pub struct IOTDataClient;

impl IOTDataClient {
    /// Raw publish method that wraps gg_request_init, gg_publish
    pub fn publish_raw(topic: &str, buffer: &[u8], read: usize) -> GGResult<()> {
        info!("topic: {}, read: {:?}, buffer: {:?}", topic, read, buffer);

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
        }
        Ok(())
    }

    /// Allows publishing a message of anything that implements AsRef<[u8]> to be published
    pub fn publish<T: AsRef<[u8]>>(topic: &str, message: T) -> GGResult<()> {
        let as_bytes = message.as_ref();
        let size = as_bytes.len();
        Self::publish_raw(topic, as_bytes, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_str() {
        let topic = "foo";
        let message = "this is my message";
        IOTDataClient::publish(topic, message).unwrap();
    }
}
