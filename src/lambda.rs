use std::ffi::CString;
use serde::Serialize;
use base64::encode;
use serde_json;
use std::os::raw::c_void;
use std::ptr;
use std::convert::TryFrom;
use std::default::Default;

use crate::bindings::*;
use crate::GGResult;
use crate::error::GGError;
use crate::with_request;
use crate::request::GGRequestResponse;

#[cfg(all(test, feature = "mock"))]
use self::mock::*;


/// Options to invoke a specified lambda
#[derive(Clone, Debug)]
pub struct InvokeOptions<C: Serialize> {
    /// The full ARN of the lambda
    pub function_arn: String,
    /// base64 json string
    pub customer_context: C,
    /// Version number of the lambda function
    pub qualifier: String,
}

impl<C: Serialize> InvokeOptions<C> {

    /// Creates a new instance of InvokeOptions
    pub fn new(function_arn: String, customer_context: C, qualifier: String) -> Self {
        InvokeOptions {
            function_arn,
            customer_context,
            qualifier,
        }
    }

    fn serialize_customer_context(&self) -> GGResult<String> {
        let json = serde_json::to_string(&self.customer_context).map_err(GGError::from)?;
        Ok(encode(json))
    }

}

/// Provides the ability to execute other lambda functions
pub struct LambdaClient {
    #[cfg(all(test, feature = "mock"))]
    pub mocks: MockHolder
}

impl LambdaClient {

    /// Allows lambda invocation with an optional payload and wait for a response.
    ///
    /// # Example
    /// ```rust
    /// use serde::Serialize;
    /// use aws_greengrass_core_rust::lambda::LambdaClient;
    /// use aws_greengrass_core_rust::lambda::InvokeOptions;
    ///
    /// #[derive(Serialize)]
    /// struct Context {
    ///     foo: String,
    ///     bar: String,
    /// }
    ///
    /// fn main() {
    ///     let payload = "Some payload";;
    ///     let context = Context { foo: "blah".to_owned(), bar: "baz".to_owned() };
    ///     let options = InvokeOptions::new("my_func_arn".to_owned(), context, "lambda qualifier".to_owned());
    ///     let response = LambdaClient::default().invoke_sync(options, Some(payload));
    ///     println!("response: {:?}", response);
    /// }
    /// ```
    #[cfg(not(feature = "mock"))]
    pub fn invoke_sync<C: Serialize, P: AsRef<[u8]>>(&self, option: InvokeOptions<C>, payload: Option<P>) -> GGResult<Option<Vec<u8>>> {
        invoke(&option, InvokeType::InvokeRequestResponse, &payload)
    }

    /// Allows lambda invocation with an optional payload. The lambda will be executed asynchronously and no response will be returned
    ///
    /// # Example
    /// ```rust
    /// use serde::Serialize;
    /// use aws_greengrass_core_rust::lambda::LambdaClient;
    /// use aws_greengrass_core_rust::lambda::InvokeOptions;
    ///
    /// #[derive(Serialize)]
    /// struct Context {
    ///     foo: String,
    ///     bar: String,
    /// }
    ///
    /// fn main() {
    ///     let payload = "Some payload";
    ///     let context = Context { foo: "blah".to_owned(), bar: "baz".to_owned() };
    ///     let options = InvokeOptions::new("my_func_arn".to_owned(), context, "lambda qualifier".to_owned());
    ///     if let Err(e) = LambdaClient::default().invoke_async(options, Some(payload)) {
    ///         eprintln!("Error occurred: {}", e);
    ///     }
    /// }
    /// ```
    #[cfg(not(feature = "mock"))]
    pub fn invoke_async<C: Serialize, P: AsRef<[u8]>>(&self, option: InvokeOptions<C>, payload: Option<P>) -> GGResult<()> {
        invoke(&option, InvokeType::InvokeEvent, &payload)
            .map(|_| ())
    }

    #[cfg(all(test, feature = "mock"))]
    pub fn invoke_sync<C: Serialize, P: AsRef<[u8]>>(&self, option: &InvokeOptions<C>, payload: &Option<P>) -> GGResult<Option<Vec<u8>>> {
        log::warn!("Mock invoke_sync is being executed!!! This should not happen in prod!!!!");
        let opts = InvokeOptionsInput::from(option);
        let payload_bytes = payload.as_ref().map(|p| p.as_ref().to_vec());
        self.mocks
            .invoke_sync_inputs
            .borrow_mut()
            .push(InvokeInput(opts, payload_bytes));

        if let Some(output) = self.mocks.invoke_sync_outputs.borrow_mut().pop() {
            output.map(|vec| serde_json::from_slice(vec.as_ref()).unwrap())
        } else {
            Ok(None)
        }
    }

    #[cfg(all(test, feature = "mock"))]
    pub fn invoke_async<C: Serialize, P: AsRef<[u8]>>(&self, option: &InvokeOptions<C>, payload: &Option<P>) -> GGResult<()> {
        log::warn!("Mock invoke_async is being executed!!! This should not happen in prod!!!!");
        let opts = InvokeOptionsInput::from(option);

        let payload_bytes = payload.as_ref().map(|p| p.as_ref().to_vec());
        self.mocks
            .invoke_async_inputs
            .borrow_mut()
            .push(InvokeInput(opts, payload_bytes));

        if let Some(output) = self.mocks.invoke_async_outputs.borrow_mut().pop() {
            output
        } else {
            Ok(())
        }
    }

    /// When the mock feature is turned on this will contain captured inputs and return
    /// provided outputs
    #[cfg(all(test, feature = "mock"))]
    pub fn with_mocks(self, mocks: MockHolder) -> Self {
        LambdaClient {mocks, ..self}
    }
}

impl Default for LambdaClient {
    fn default() -> Self {
        LambdaClient {
            #[cfg(all(test, feature = "mock"))]
            mocks: MockHolder::default(),
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InvokeType {
    /// Invoke the function asynchronously
    InvokeEvent,
    /// Invoke the function synchronously (default)
    InvokeRequestResponse,
}

impl TryFrom<gg_invoke_type> for InvokeType {
    type Error = GGError;

    #[allow(non_upper_case_globals)]
    fn try_from(value: gg_invoke_type) -> Result<Self, Self::Error> {
        match value {
            gg_invoke_type_GG_INVOKE_EVENT => Ok(Self::InvokeEvent),
            gg_invoke_type_GG_INVOKE_REQUEST_RESPONSE => Ok(Self::InvokeRequestResponse),
            _ => Err(GGError::Unknown(format!("Unknown invoke type: {}", value)))
        }
    }
}

impl Default for InvokeType {
    fn default() -> Self {
        InvokeType::InvokeEvent
    }
}

impl InvokeType {
    fn as_c_invoke_type(&self) -> gg_invoke_type {
        match *self {
            Self::InvokeEvent => gg_invoke_type_GG_INVOKE_EVENT,
            Self::InvokeRequestResponse => gg_invoke_type_GG_INVOKE_REQUEST_RESPONSE,
        }
    }
}

fn invoke<C: Serialize, P: AsRef<[u8]>>(option: &InvokeOptions<C>, invoke_type: InvokeType, payload: &Option<P>) -> GGResult<Option<Vec<u8>>> {
    unsafe {
        let function_arn_c =  CString::new(option.function_arn.as_str()).map_err(GGError::from)?;
        let customer_context_c = CString::new(option.serialize_customer_context()?).map_err(GGError::from)?;
        let qualifier_c = CString::new(option.qualifier.as_str()).map_err(GGError::from)?;
        let payload_bytes = payload.as_ref().map(|p| p.as_ref());
        let (payload_c, payload_size) = if let Some(p) = payload_bytes {
            (p as *const _ as *const c_void, p.len())
        } else {
            (ptr::null(), 0)
        };

        let options_c = Box::new(gg_invoke_options {
            function_arn: function_arn_c.as_ptr(),
            customer_context: customer_context_c.as_ptr(),
            qualifier: qualifier_c.as_ptr(),
            type_: invoke_type.as_c_invoke_type(),
            payload: payload_c,
            payload_size,
        });

        let mut req: gg_request = ptr::null_mut();
        with_request!(req, {
            let mut res = gg_request_result {
                request_status: gg_request_status_GG_REQUEST_SUCCESS,
            };
            let invoke_res = gg_invoke(
                req,
                Box::into_raw(options_c),
                &mut res,
            );
            GGError::from_code(invoke_res)?;

            match invoke_type {
                InvokeType::InvokeEvent => {
                   GGRequestResponse::try_from(&res)?.to_error_result(req)?;
                   Ok(None)
                }
                InvokeType::InvokeRequestResponse => GGRequestResponse::try_from(&res)?.read(req),
            }
        })
    }
}

/// Provides mock testing utilities
#[cfg(all(test, feature = "mock"))]
pub mod mock {
    use super::*;
    use std::cell::RefCell;

    #[derive(Clone)]
    pub struct InvokeOptionsInput {
        /// The full ARN of the lambda
        pub function_arn: String,
        /// base64 json string
        pub customer_context: Vec<u8>,
        /// Version number of the lambda function
        pub qualifier: String,
    }

    impl<C:Serialize> From<&InvokeOptions<C>> for InvokeOptionsInput {
        fn from(opts: &InvokeOptions<C>) -> Self {
            InvokeOptionsInput {
                function_arn: opts.function_arn.to_owned(),
                customer_context: serde_json::to_vec(&opts.customer_context).unwrap(),
                qualifier: opts.qualifier.to_owned(),
            }
        }
    }

    #[derive(Clone)]
    pub struct InvokeInput(pub InvokeOptionsInput, pub Option<Vec<u8>>);

    /// used to override input and output when the mock feature is enabled
    pub struct MockHolder {
        pub invoke_sync_inputs: RefCell<Vec<InvokeInput>>,
        pub invoke_sync_outputs: RefCell<Vec<GGResult<Vec<u8>>>>,
        pub invoke_async_inputs: RefCell<Vec<InvokeInput>>,
        pub invoke_async_outputs: RefCell<Vec<GGResult<()>>>,
    }

    impl MockHolder {

        pub fn with_invoke_sync_outputs(self, invoke_sync_outputs: Vec<GGResult<Vec<u8>>>) -> Self {
            MockHolder {
                invoke_sync_outputs: RefCell::new(invoke_sync_outputs),
                ..self
            }
        }

        pub fn with_invoke_async_outputs(self, invoke_async_outputs: Vec<GGResult<()>>) -> Self {
            MockHolder {
                invoke_async_outputs: RefCell::new(invoke_async_outputs),
                ..self
            }
        }

    }

    impl Default for MockHolder {
        fn default() -> Self {
            MockHolder {
                invoke_sync_inputs: RefCell::new(vec![]),
                invoke_sync_outputs: RefCell::new(vec![]),
                invoke_async_inputs: RefCell::new(vec![]),
                invoke_async_outputs: RefCell::new(vec![]),
            }
        }
    }

    impl Clone for MockHolder {
        fn clone(&self) -> Self {
            MockHolder {
                invoke_sync_inputs: RefCell::new(self.invoke_sync_inputs.borrow().clone()),
                invoke_async_inputs: RefCell::new(self.invoke_async_inputs.borrow().clone()),
                invoke_sync_outputs: RefCell::new(vec![]),
                invoke_async_outputs: RefCell::new(vec![]),
            }
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Clone)]
    struct TestContext {
        foo: String
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    struct TestPayload {
        msg: String
    }

    //noinspection DuplicatedCode
    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_invoke_async() {
        reset_test_state();

        let function_arn = "function_arn2323";

        let context = TestContext {
            foo: "bar".to_string()
        };

        let payload = TestPayload {
            msg: "The message of my payload".to_owned()
        };

        let qualifier = "12121221";

        let payload_bytes = serde_json::to_vec(&payload).unwrap();

        let options =
            InvokeOptions::new(function_arn.to_owned(), context.clone(), qualifier.to_owned());

        LambdaClient::default().invoke_async(options, Some(payload_bytes.clone())).unwrap();

        GG_INVOKE_ARGS.with(|rc| {
            let args = rc.borrow();
            assert_eq!(args.qualifier, qualifier);
            assert_eq!(args.payload, payload_bytes);
            assert_eq!(args.customer_context, serde_json::to_vec(&context).unwrap());
            assert_eq!(args.function_arn, function_arn);
            assert_eq!(args.invoke_type, InvokeType::InvokeEvent);
        });

        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

    //noinspection DuplicatedCode
    #[cfg(not(feature = "mock"))]
    #[test]
    fn test_invoke_sync() {
        reset_test_state();
        let response = TestPayload {
            msg: "This is the sync response!".to_owned()
        };
        GG_REQUEST_READ_BUFFER.with(|rc| {
            let bytes = serde_json::to_vec(&response).unwrap();
            rc.replace(bytes);
        });

        let function_arn = "function_arn2323t867";

        let context = TestContext {
            foo: "bark".to_string()
        };

        let payload = TestPayload {
            msg: "The message of my payloadkhkjb".to_owned()
        };

        let qualifier = "12121221";

        let payload_bytes = serde_json::to_vec(&payload).unwrap();

        let options =
            InvokeOptions::new(function_arn.to_owned(), context.clone(), qualifier.to_owned());

        let result: Vec<u8> = LambdaClient::default().invoke_sync(options, Some(payload_bytes.clone())).unwrap().unwrap();
        assert_eq!(serde_json::from_slice::<TestPayload>(result.as_ref()).unwrap(), response);

        GG_INVOKE_ARGS.with(|rc| {
            let args = rc.borrow();
            assert_eq!(args.qualifier, qualifier);
            assert_eq!(args.payload, payload_bytes);
            assert_eq!(args.customer_context, serde_json::to_vec(&context).unwrap());
            assert_eq!(args.function_arn, function_arn);
            assert_eq!(args.invoke_type, InvokeType::InvokeRequestResponse);
        });

        GG_CLOSE_REQUEST_COUNT.with(|rc| assert_eq!(*rc.borrow(), 1));
        GG_REQUEST.with(|rc| assert!(!rc.borrow().is_default()));
    }

}