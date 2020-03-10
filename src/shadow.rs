use serde_json::{self, Value};
use std::convert::TryFrom;
use std::ffi::CString;
use std::ptr;

use crate::bindings::*;
use crate::error::GGError;
use crate::request::{read_response_data, ErrorResponse, GGRequestResponse};
use crate::try_clean;
use crate::GGResult;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[cfg(all(test, feature = "mock"))]
use self::mock::*;
use std::borrow::BorrowMut;

#[derive(Clone)]
pub struct ShadowClient {
    /// When the mock feature is turned on this field will contain captured input
    /// and values to be returned
    #[cfg(all(test, feature = "mock"))]
    pub mocks: MockHolder,
}

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
    /// if let Ok(maybe_json) = ShadowClient::default().get_thing_shadow::<Value>("my_thing") {
    ///     println!("Retrieved: {:?}", maybe_json);
    /// }
    /// ```
    #[cfg(not(all(test, feature = "mock")))]
    pub fn get_thing_shadow<'a, T: DeserializeOwned>(
        &self,
        thing_name: &str,
    ) -> GGResult<Option<T>> {
        let (bytes, response) = read_thing_shadow(thing_name)?;
        // First check to see if the response contains an error
        // This might be a bit inefficient, but I couldn't think of a better way to do it at the time
        // as type T could just be Value or another type that would be successful in parsing, making the API inconsistent
        if let Ok(err_response) = serde_json::from_slice::<ErrorResponse>(&bytes) {
            match err_response.code {
                404 => Ok(None),
                _ => Err(GGError::Unknown(format!(
                    "code: {}, message: {}",
                    err_response.code, err_response.message
                ))),
            }
        } else {
            let json: T = serde_json::from_slice(&bytes).map_err(GGError::from)?;
            Ok(Some(json))
        }
    }

    /// Updates a shadow thing with the specified document.
    ///
    /// # Arguments
    ///
    /// * `thing_name` - The name of the device to update the shadow document
    /// * `doc` - Json serializable content to update
    #[cfg(not(all(test, feature = "mock")))]
    pub fn update_thing_shadow<T: Serialize>(&self, thing_name: &str, doc: &T) -> GGResult<()> {
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
            try_clean!(req, GGError::from_code(update_res));

            let response = try_clean!(req, GGRequestResponse::try_from(&res));
            if response.is_error() {
                let response_bytes = try_clean!(req, read_response_data(req));
                let error_response =
                    try_clean!(req, ErrorResponse::try_from(response_bytes.as_slice()));
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
    #[cfg(not(all(test, feature = "mock")))]
    pub fn delete_thing_shadow(&self, thing_name: &str) -> GGResult<()> {
        unsafe {
            let mut req: gg_request = ptr::null_mut();
            let req_init = gg_request_init(&mut req);
            GGError::from_code(req_init)?;

            let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;

            let mut res_c = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };

            let delete_res = gg_delete_thing_shadow(req, thing_name_c.as_ptr(), &mut res_c);
            try_clean!(req, GGError::from_code(req_init));

            let response = try_clean!(req, GGRequestResponse::try_from(&res_c));
            if response.is_error() {
                let resp_bytes = try_clean!(req, read_response_data(req));
                let err_resp = try_clean!(req, ErrorResponse::try_from(resp_bytes.as_slice()));
                let response_2 = response.with_error_response(Some(err_resp));
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
    pub fn get_thing_shadow<'a, T: DeserializeOwned>(
        &self,
        thing_name: &str,
    ) -> GGResult<Option<T>> {
        self.mocks
            .get_shadow_thing_inputs
            .borrow_mut()
            .push(GetShadowThingInput(thing_name.to_owned()));
        if let Some(output) = self.mocks.get_shadow_thing_outputs.borrow_mut().pop() {
            output.map(|o| serde_json::from_slice::<T>(o.as_slice()).ok())
        } else {
            Ok(serde_json::from_str(DEFAULT_SHADOW_DOC).ok())
        }
    }

    #[cfg(all(test, feature = "mock"))]
    pub fn update_thing_shadow<T: Serialize>(&self, thing_name: &str, doc: &T) -> GGResult<()> {
        let bytes = serde_json::to_vec(doc).map_err(GGError::from)?;
        self.mocks
            .update_thing_shadow_inputs
            .borrow_mut()
            .push(UpdateThingShadowInput(thing_name.to_owned(), bytes));
        if let Some(output) = self.mocks.update_thing_shadow_outputs.borrow_mut().pop() {
            output
        } else {
            Ok(())
        }
    }

    #[cfg(all(test, feature = "mock"))]
    pub fn delete_thing_shadow(&self, thing_name: &str) -> GGResult<()> {
        self.mocks
            .delete_thing_shadow_inputs
            .borrow_mut()
            .push(DeleteThingShadowInput(thing_name.to_owned()));
        if let Some(output) = self.mocks.delete_thing_shadow_outputs.borrow_mut().pop() {
            output
        } else {
            Ok(())
        }
    }
}

impl Default for ShadowClient {
    fn default() -> Self {
        ShadowClient {
            #[cfg(all(test, feature = "mock"))]
            mocks: MockHolder::default(),
        }
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

#[cfg(all(test, feature = "mock"))]
pub mod mock {
    use crate::GGResult;
    use serde::Serialize;
    use std::cell::RefCell;

    pub const DEFAULT_SHADOW_DOC: &'static str = r#"{
    "state" : {
        "desired" : {
          "color" : "RED",
          "sequence" : [ "RED", "GREEN", "BLUE" ]
        },
        "reported" : {
          "color" : "GREEN"
        }
    },
    "metadata" : {
        "desired" : {
            "color" : {
                "timestamp" : 12345
            },
            "sequence" : {
                "timestamp" : 12345
            }
        },
        "reported" : {
            "color" : {
                "timestamp" : 12345
            }
        }
    },
    "version" : 10,
    "clientToken" : "UniqueClientToken",
    "timestamp": 123456789
}"#;

    #[derive(Debug, Clone)]
    pub struct GetShadowThingInput(pub String);
    /// second parameter is serde serialized parameter
    #[derive(Debug, Clone)]
    pub struct UpdateThingShadowInput(pub String, pub Vec<u8>);
    #[derive(Debug, Clone)]
    pub struct DeleteThingShadowInput(pub String);

    /// Used to hold inputs and override default outputs for mocks
    pub struct MockHolder {
        pub get_shadow_thing_inputs: RefCell<Vec<GetShadowThingInput>>,
        /// By providing the byte vec we can get around the generic type signature
        /// issue. This should be Deserializable to the type parameter passed to the method
        pub get_shadow_thing_outputs: RefCell<Vec<GGResult<Vec<u8>>>>,
        /// Serializable representation to get past generics issue
        pub update_thing_shadow_inputs: RefCell<Vec<UpdateThingShadowInput>>,
        pub update_thing_shadow_outputs: RefCell<Vec<GGResult<()>>>,
        pub delete_thing_shadow_inputs: RefCell<Vec<DeleteThingShadowInput>>,
        pub delete_thing_shadow_outputs: RefCell<Vec<GGResult<()>>>,
    }

    impl Clone for MockHolder {
        fn clone(&self) -> Self {
            MockHolder {
                get_shadow_thing_inputs: self.get_shadow_thing_inputs.clone(),
                update_thing_shadow_inputs: self.update_thing_shadow_inputs.clone(),
                delete_thing_shadow_inputs: self.delete_thing_shadow_inputs.clone(),
                // NOTE: Cannot clone outputs. Keep this in mind in tests
                get_shadow_thing_outputs: RefCell::new(vec![]),
                update_thing_shadow_outputs: RefCell::new(vec![]),
                delete_thing_shadow_outputs: RefCell::new(vec![]),
            }
        }
    }

    impl Default for MockHolder {
        fn default() -> Self {
            MockHolder {
                get_shadow_thing_inputs: RefCell::new(vec![]),
                update_thing_shadow_inputs: RefCell::new(vec![]),
                delete_thing_shadow_inputs: RefCell::new(vec![]),
                get_shadow_thing_outputs: RefCell::new(vec![]),
                update_thing_shadow_outputs: RefCell::new(vec![]),
                delete_thing_shadow_outputs: RefCell::new(vec![]),
            }
        }
    }

    // Note: This is to get past compile issues.. Mock testing for threads
    // could result in undefined behavior
    unsafe impl Send for MockHolder {}
    unsafe impl Sync for MockHolder {}
}
