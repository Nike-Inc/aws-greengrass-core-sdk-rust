/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

use serde_json;
use std::convert::TryFrom;
use std::ffi::CString;
use std::ptr;

use crate::bindings::*;
use crate::error::GGError;
use crate::request::GGRequestResponse;
use crate::with_request;
use crate::GGResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::default::Default;

#[cfg(all(test, feature = "mock"))]
use self::mock::*;

/// Provides the ability to interact with a Thing's (Device) Shadow document
///
/// Information on shadow documents can be found at: https://docs.aws.amazon.com/iot/latest/developerguide/device-shadow-document.html#device-shadow-example
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
    pub fn get_thing_shadow<T: DeserializeOwned>(&self, thing_name: &str) -> GGResult<Option<T>> {
        if let Some(bytes) = read_thing_shadow(thing_name)? {
            let json: T = serde_json::from_slice(&bytes).map_err(GGError::from)?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    /// Updates a shadow thing with the specified document.
    ///
    /// # Arguments
    ///
    /// * `thing_name` - The name of the device to update the shadow document
    /// * `doc` - Json serializable content to update
    ///
    /// # Examples
    /// ```rust
    /// use serde::Serialize;
    /// use aws_greengrass_core_rust::shadow::ShadowClient;
    ///
    /// #[derive(Serialize)]
    /// struct MyStruct;
    ///
    /// let result = ShadowClient::default().update_thing_shadow("foo", &MyStruct);
    /// ```
    #[cfg(not(all(test, feature = "mock")))]
    pub fn update_thing_shadow<T: Serialize>(&self, thing_name: &str, doc: &T) -> GGResult<()> {
        let json_string = serde_json::to_string(doc).map_err(GGError::from)?;
        unsafe {
            let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;
            let json_string_c = CString::new(json_string).map_err(GGError::from)?;
            let mut req: gg_request = ptr::null_mut();
            with_request!(req, {
                let mut res = gg_request_result {
                    request_status: gg_request_status_GG_REQUEST_SUCCESS,
                };
                let update_res = gg_update_thing_shadow(
                    req,
                    thing_name_c.as_ptr(),
                    json_string_c.as_ptr(),
                    &mut res,
                );
                GGError::from_code(update_res)?;
                GGRequestResponse::try_from(&res)?.to_error_result(req)
            })
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
            let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;
            let mut req: gg_request = ptr::null_mut();
            with_request!(req, {
                let mut res_c = gg_request_result {
                    request_status: gg_request_status_GG_REQUEST_SUCCESS,
                };
                let delete_res = gg_delete_thing_shadow(req, thing_name_c.as_ptr(), &mut res_c);
                GGError::from_code(delete_res)?;
                GGRequestResponse::try_from(&res_c)?.to_error_result(req)
            })
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
            Ok(serde_json::from_str(self::test::DEFAULT_SHADOW_DOC).ok())
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

fn read_thing_shadow(thing_name: &str) -> GGResult<Option<Vec<u8>>> {
    unsafe {
        let thing_name_c = CString::new(thing_name).map_err(GGError::from)?;
        let mut req: gg_request = ptr::null_mut();
        with_request!(req, {
            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };
            let fetch_res = gg_get_thing_shadow(req, thing_name_c.as_ptr(), &mut res);
            GGError::from_code(fetch_res)?;
            GGRequestResponse::try_from(&res)?.read(req)
        })
    }
}

#[cfg(all(test, feature = "mock"))]
pub mod mock {
    use crate::GGResult;
    use serde::Serialize;
    use std::cell::RefCell;

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

#[cfg(test)]
pub mod test {
    use super::*;
    use serde_json::Value;

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

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_get_shadow_thing() {
        reset_test_state();
        GG_REQUEST_READ_BUFFER.with(|rc| rc.replace(DEFAULT_SHADOW_DOC.as_bytes().to_vec()));
        let thing_name = "my_thing_get";
        let shadow = ShadowClient::default()
            .get_thing_shadow::<Value>(thing_name)
            .unwrap()
            .unwrap();
        GG_SHADOW_THING_ARG.with(|rc| assert_eq!(*rc.borrow(), thing_name));
        assert_eq!(
            shadow,
            serde_json::from_str::<Value>(DEFAULT_SHADOW_DOC).unwrap()
        );
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_delete_shadow_thing() {
        reset_test_state();
        let thing_name = "my_thing_get_delete";
        ShadowClient::default()
            .delete_thing_shadow(thing_name)
            .unwrap();
        GG_SHADOW_THING_ARG.with(|rc| assert_eq!(*rc.borrow(), thing_name));
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_update_shadow_thing() {
        reset_test_state();
        let thing_name = "my_thing_update";
        let doc = serde_json::from_str::<Value>(DEFAULT_SHADOW_DOC).unwrap();
        ShadowClient::default()
            .update_thing_shadow(thing_name, &doc)
            .unwrap();
        GG_SHADOW_THING_ARG.with(|rc| assert_eq!(*rc.borrow(), thing_name));
        GG_UPDATE_PAYLOAD.with(|rc| {
            assert_eq!(*rc.borrow(), serde_json::to_string(&doc).unwrap());
        });
        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }
}
