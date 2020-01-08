include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Request {
    topic: String;
}

impl Request {

    pub new(topic: &str) -> Self {
        Request {
            topic: topic.to_owned();
        }
    }

    pub publish(buf: &u8, size: usize) -> Result<(), GGError> {
        unsafe {
            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;
    
            let topic_c = CString::new(topic)?;
            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };
    
            let pub_res = gg_publish(req, topic_c.as_ptr(), message as *const _ as *const c_void, size, &mut res);
            GGError::from_code(pub_res)?;

            let close_res = gg_request_close(req);
            GGError::from_code(close_res)?;
        }
        Ok(())
    }
}