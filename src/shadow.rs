use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use serde_json::{self, Value};
use std::convert::TryFrom;

use crate::bindings::*;
use crate::request::{GGRequestResponse, read_response_data};
use crate::GGResult;
use crate::error::GGError;

#[derive(Debug, Clone)]
pub struct ShadowThing {
    //todo -- change me
    pub json: Value,
}

pub struct ShadowClient;

impl ShadowClient {

    pub fn get_thing_shadow(thing_name: &str) -> GGResult<(ShadowThing, GGRequestResponse)> {
        let (bytes, response) = read_shadow_thing(thing_name)?;
        let json: Value = serde_json::from_slice(&bytes).map_err(GGError::from)?;
        let thing = ShadowThing {
            json
        };
        Ok((thing,response))
    }

}

fn read_shadow_thing(thing_name: &str) -> GGResult<(Vec<u8>, GGRequestResponse)> {
    unsafe {
        let mut req: gg_request = ptr::null_mut();
        let req_init = gg_request_init(&mut req);
        GGError::from_code(req_init)?;

        let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;

        let mut res = gg_request_result {
            request_status: gg_request_status_GG_REQUEST_SUCCESS,
        };

        let fetch_res = gg_get_thing_shadow(req, thing_name_c.as_ptr(), &mut res);
        GGError::from_code(fetch_res)?;

        let read_res = read_response_data(req);

        let close_res = gg_request_close(req);
        GGError::from_code(close_res)?;

        let converted_response = GGRequestResponse::try_from(&res)?;
        read_res.map(|res| (res, converted_response))
    }
}