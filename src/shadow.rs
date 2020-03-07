use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use serde_json::{self, Value};
use std::convert::TryFrom;

use crate::bindings::*;
use crate::request::{GGRequestResponse, read_response_data};
use crate::GGResult;
use crate::error::GGError;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::default::Default;

pub struct ShadowClient;

impl ShadowClient {

    /// Get thing shadow for thing name.
    ///
    /// # Arguments
    ///
    /// * `thing_name` - The name of the device for the thing shadow to get
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde_json::Value;
    /// use aws_greengrass_core_rust::shadow::ShadowClient;
    ///
    /// if let Ok((json, res)) = ShadowClient::default().get_thing_shadow::<Value>("my_thing") {
    ///     println!("Retrieved: {:?}", json);
    /// }
    /// ```
    pub fn get_thing_shadow<'a, T: DeserializeOwned>(&self, thing_name: &str) -> GGResult<(T, GGRequestResponse)> {
        let (bytes, response) = read_thing_shadow(thing_name)?;
        let json: T = serde_json::from_slice(&bytes).map_err(GGError::from)?;
        Ok((json,response))
    }

    /// Updates a shadow thing with the specified document.
    ///
    /// # Arguments
    ///
    /// * `thing_name` - The name of the device to update the shadow document
    /// * `doc` - Json serializable content to update
    pub fn update_thing_shadow<T: Serialize>(&self, thing_name: &str, doc: &T) -> GGResult<GGRequestResponse> {
        let json_string = serde_json::to_string(doc).map_err(GGError::from)?;
        unsafe {
            let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;
            let json_string_c = CString::new(json_string).map_err(GGError::from)?;

            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;

            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };

            let update_res = gg_update_thing_shadow(
                req,
                thing_name_c.as_ptr(),
                json_string_c.as_ptr(),
                &mut res,
            );

            let close_res = gg_request_close(req);
            GGError::from_code(close_res)?;

            GGRequestResponse::try_from(&res)
        }
    }

    /// Deletes thing shadow for thing name.
    ///
    /// # Arguments
    ///
    /// * `thing_name` - The name of the device for the thing shadow to get
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde_json::Value;
    /// use aws_greengrass_core_rust::shadow::ShadowClient;
    ///
    /// let res = ShadowClient::default().delete_thing_shadow("my_thing");
    /// ```
    pub fn delete_thing_shadow(&self, thing_name: &str) -> GGResult<GGRequestResponse> {
        unsafe {
            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;

            let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;

            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };

            let delete_res = gg_delete_thing_shadow(req, thing_name_c.as_ptr(), &mut res);
            GGError::from_code(req_init)?;

            let close_res = gg_request_close(req);
            GGError::from_code(close_res)?;

            GGRequestResponse::try_from(&res)
        }
    }

}

impl Default for ShadowClient {
    fn default() -> Self {
        ShadowClient
    }
}

fn read_thing_shadow(thing_name: &str) -> GGResult<(Vec<u8>, GGRequestResponse)> {
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