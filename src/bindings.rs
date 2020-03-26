#![allow(dead_code, improper_ctypes, unused_variables, non_upper_case_globals, non_camel_case_types,
    non_snake_case, clippy::all)]
//! This module encapsulates the bindings for the C library
//! The bindings are regenerated on build on every build.
//! For testing we do two things
//!
//! 1. Use a mocked version with test hooks for the rest of the project
//! 2. Add another module so the tests against the generated bindings is still run
//!
//! improper c_types is ignored. This is do to the u128 issue described here: https://github.com/rust-lang/rust-bindgen/issues/1549
//! dead_code is allowed, do to a number of things in the bindings not being used

#[cfg(all(not(test), not(feature = "coverage")))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(any(test, feature = "coverage"))]
pub use self::test::*;

/// Provides stubbed testing versions of methods, etc that match greengrasssdk.h
/// Useful for internal testing.
/// All test that utilize this package must have a #[cfg(not(feature = "mock"))] or the build will fail.
#[cfg(any(test, feature = "coverage"))]
pub mod test {
    use crate::lambda::InvokeType;
    use base64;
    use std::cell::RefCell;
    use std::convert::TryFrom;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_void;
    use std::thread_local;
    use std::sync::Mutex;
    use uuid::Uuid;
    use lazy_static::lazy_static;
    use crate::runtime::EventContext;

    // For things that won't work within a thread local. Attempt to use a thread local when possible
    // since tests are run in parallel. Generally only stuff related to testing runtime package should be in here
    // brecause the handlers are run in a separate thread
    lazy_static! {
        // This could problems if more than than one test is accessing. Try to limit usage.
        pub(crate) static ref GG_HANDLER: Mutex<gg_lambda_handler> = Mutex::new(None);
        pub(crate) static ref GG_LAMBDA_HANDLER_RESPONSE: Mutex<Vec<u8>> = Mutex::new(vec![]);
        pub(crate) static ref GG_LAMBDA_HANDLER_ERR_RESPONSE: Mutex<Vec<u8>> = Mutex::new(vec![]);
    }

    // Thread locals used for testing
    thread_local! {
        pub(crate) static GG_SHADOW_THING_ARG: RefCell<String> = RefCell::new("".to_owned());
        pub(crate) static GG_UPDATE_PAYLOAD: RefCell<String> = RefCell::new("".to_owned());
        pub(crate) static GG_REQUEST_READ_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![]);
        pub(crate) static GG_REQUEST: RefCell<_gg_request> = RefCell::new(_gg_request::default());
        pub(crate) static GG_LAMBDA_HANDLER_READ_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![]);
        /// used to store the arguments passed to gg_publish
        pub(crate) static GG_PUBLISH_ARGS: RefCell<GGPublishPayloadArgs> = RefCell::new(GGPublishPayloadArgs::default());
        pub(crate) static GG_PUBLISH_WITH_OPTIONS_ARGS: RefCell<GGPublishPayloadArgs> = RefCell::new(GGPublishPayloadArgs::default());
        pub(crate) static GG_GET_SECRET_VALUE_ARGS: RefCell<GGGetSecretValueArgs> = RefCell::new(GGGetSecretValueArgs::default());
        pub(crate) static GG_GET_SECRET_VALUE_RETURN: RefCell<gg_error> = RefCell::new(gg_error_GGE_SUCCESS);
        pub(crate) static GG_CLOSE_REQUEST_COUNT: RefCell<u8> = RefCell::new(0);
        pub(crate) static GG_PUBLISH_OPTION_INIT_COUNT: RefCell<u8> = RefCell::new(0);
        pub(crate) static GG_PUBLISH_OPTION_FREE_COUNT: RefCell<u8> = RefCell::new(0);
        pub(crate) static GG_INVOKE_ARGS: RefCell<GGInvokeArgs> = RefCell::new(GGInvokeArgs::default());
        pub(crate) static GG_PUBLISH_OPTIONS_SET_QUEUE_FULL_POLICY: RefCell<gg_queue_full_policy_options> = RefCell::new(1515);
        pub(crate) static GG_LOG_ARGS: RefCell<Vec<LogArgs>> = RefCell::new(vec![]);
    }

    pub fn reset_test_state() {
        GG_SHADOW_THING_ARG.with(|rc| rc.replace("".to_owned()));
        GG_UPDATE_PAYLOAD.with(|rc| rc.replace("".to_owned()));
        GG_REQUEST_READ_BUFFER.with(|rc| rc.replace(vec![]));
        GG_REQUEST.with(|rc| rc.replace(_gg_request::default()));
        GG_LAMBDA_HANDLER_READ_BUFFER.with(|rc| rc.replace(vec![]));
        GG_PUBLISH_ARGS.with(|rc| rc.replace(GGPublishPayloadArgs::default()));
        GG_PUBLISH_WITH_OPTIONS_ARGS.with(|rc| rc.replace(GGPublishPayloadArgs::default()));
        GG_GET_SECRET_VALUE_ARGS.with(|rc| rc.replace(GGGetSecretValueArgs::default()));
        GG_CLOSE_REQUEST_COUNT.with(|rc| rc.replace(0));
        GG_PUBLISH_OPTION_INIT_COUNT.with(|rc| rc.replace(0));
        GG_PUBLISH_OPTION_FREE_COUNT.with(|rc| rc.replace(0));
        GG_GET_SECRET_VALUE_RETURN.with(|rc| rc.replace(gg_error_GGE_SUCCESS));
        GG_PUBLISH_OPTIONS_SET_QUEUE_FULL_POLICY.with(|rc| rc.replace(1515));
        GG_LOG_ARGS.with(|rc| rc.replace(vec![]));
    }

    pub(crate) fn reset_test_statics() {
        let mut handler = GG_HANDLER.lock().expect("could not acquire lock on GG_HANDLER");
        *handler = None;
        let mut handler_resp = GG_LAMBDA_HANDLER_RESPONSE.lock().expect("could not acquire lock on GG_LAMBDA_HANDLER_RESPONSE");
        *handler_resp = vec![];
        let mut handler_err_resp = GG_LAMBDA_HANDLER_ERR_RESPONSE.lock().expect("could not acquire lock on GG_LAMBDA_HANDLER_RESPONSE");
        *handler_err_resp = vec![];
    }

    #[derive(Debug, Copy, Clone, Default)]
    pub struct _gg_request {
        id: Option<Uuid>,
    }

    impl _gg_request {
        pub fn is_default(&self) -> bool {
            self.id.is_none()
        }
    }

    pub type gg_request = *mut _gg_request;
    pub const gg_request_status_GG_REQUEST_SUCCESS: gg_request_status = 0;
    pub const gg_request_status_GG_REQUEST_HANDLED: gg_request_status = 1;
    pub const gg_request_status_GG_REQUEST_UNHANDLED: gg_request_status = 2;
    pub const gg_request_status_GG_REQUEST_UNKNOWN: gg_request_status = 3;
    pub const gg_request_status_GG_REQUEST_AGAIN: gg_request_status = 4;
    pub const gg_request_status_GG_REQUEST_RESERVED_MAX: gg_request_status = 5;
    pub const gg_request_status_GG_REQUEST_RESERVED_PAD: gg_request_status = 2147483647;
    pub type gg_request_status = u32;
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct gg_request_result {
        pub request_status: gg_request_status,
    }

    pub const gg_error_GGE_SUCCESS: gg_error = 0;
    pub const gg_error_GGE_OUT_OF_MEMORY: gg_error = 1;
    pub const gg_error_GGE_INVALID_PARAMETER: gg_error = 2;
    pub const gg_error_GGE_INVALID_STATE: gg_error = 3;
    pub const gg_error_GGE_INTERNAL_FAILURE: gg_error = 4;
    pub const gg_error_GGE_TERMINATE: gg_error = 5;
    pub const gg_error_GGE_RESERVED_MAX: gg_error = 6;
    pub const gg_error_GGE_RESERVED_PAD: gg_error = 2147483647;
    pub type gg_error = u32;

    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_BEST_EFFORT:
        gg_queue_full_policy_options = 0;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_ALL_OR_ERROR:
        gg_queue_full_policy_options = 1;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_RESERVED_MAX:
        gg_queue_full_policy_options = 2;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_RESERVED_PAD:
        gg_queue_full_policy_options = 2147483647;

    pub type gg_queue_full_policy_options = u32;

    #[derive(Debug, Copy, Clone)]
    pub struct _gg_publish_options {
        _unused: [u8; 0],
    }

    pub type gg_publish_options = *mut _gg_publish_options;

    pub const gg_log_level_GG_LOG_RESERVED_NOTSET: gg_log_level = 0;
    pub const gg_log_level_GG_LOG_DEBUG: gg_log_level = 1;
    pub const gg_log_level_GG_LOG_INFO: gg_log_level = 2;
    pub const gg_log_level_GG_LOG_WARN: gg_log_level = 3;
    pub const gg_log_level_GG_LOG_ERROR: gg_log_level = 4;
    pub const gg_log_level_GG_LOG_FATAL: gg_log_level = 5;
    pub const gg_log_level_GG_LOG_RESERVED_MAX: gg_log_level = 6;
    pub const gg_log_level_GG_LOG_RESERVED_PAD: gg_log_level = 2147483647;

    pub type gg_log_level = u32;

    pub extern "C" fn gg_global_init(opt: u32) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    #[derive(PartialEq, Debug)]
    pub struct LogArgs {
        level: gg_log_level,
        format: String,
    }

    impl LogArgs {
        pub fn new(level: gg_log_level, format: &str) -> Self {
            LogArgs {
                level,
                format: format.to_owned(),
            }
        }
    }

    pub extern "C" fn gg_log(
        level: gg_log_level,
        format: *const ::std::os::raw::c_char,
    ) -> gg_error {
        unsafe {
            let format = CStr::from_ptr(format).to_owned().into_string().unwrap();
            let args = LogArgs { level, format };
            GG_LOG_ARGS.with(|rc| rc.borrow_mut().push(args));
        }
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_request_init(ggreq: *mut gg_request) -> gg_error {
        unsafe {
            let req = _gg_request {
                id: Some(Uuid::new_v4()),
            };
            GG_REQUEST.with(|rc| {
                rc.replace(req);
                std::ptr::replace(ggreq, rc.as_ptr())
            });
        }
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_request_close(ggreq: gg_request) -> gg_error {
        GG_CLOSE_REQUEST_COUNT.with(|rc| {
            let new_value = *rc.borrow() + 1;
            rc.replace(new_value);
        });

        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_request_read(
        ggreq: gg_request,
        buffer: *mut ::std::os::raw::c_void,
        buffer_size: usize,
        amount_read: *mut usize,
    ) -> gg_error {
        unsafe {
            GG_REQUEST_READ_BUFFER.with(|b| {
                let mut borrowed = b.borrow().clone();
                // If the vector is empty, don't do anything and notify that we read zero bytes
                if borrowed.is_empty() {
                    amount_read.write(0);
                } else {
                    // Find the index to split the array off at
                    let index = if buffer_size > borrowed.len() {
                        borrowed.len()
                    } else {
                        buffer_size
                    };
                    // borrowed will now contain everything up to index
                    let next = borrowed.split_off(index);
                    println!("gg_request_read: writing buffer: {:?}", borrowed);
                    buffer.copy_from_nonoverlapping(
                        borrowed.as_ptr() as *const c_void,
                        borrowed.len(),
                    );
                    amount_read.write(borrowed.len());
                    // replace the refcell with the rest of the vec
                    b.replace(next);
                }
            });
            gg_error_GGE_SUCCESS
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct gg_lambda_context {
        pub function_arn: *const ::std::os::raw::c_char,
        pub client_context: *const ::std::os::raw::c_char,
    }

    pub type gg_lambda_handler =
        ::std::option::Option<unsafe extern "C" fn(cxt: *const gg_lambda_context)>;

    pub extern "C" fn gg_runtime_start(handler: gg_lambda_handler, opt: u32) -> gg_error {
        let mut current_handler = GG_HANDLER.lock().unwrap();
        *current_handler = handler;
        gg_error_GGE_SUCCESS
    }

    /// Sets up the GG_LAMBDA_HANDLER_READ_BUFFER and calls the registered c handler
    pub(crate) fn send_to_handler(event_ctx: EventContext) {
        let EventContext(event, ctx) = event_ctx;
        GG_LAMBDA_HANDLER_READ_BUFFER.with(|rc| rc.replace(event));
        let locked = GG_HANDLER.lock().unwrap();
        if let Some(handler) = *locked {
            unsafe {
                let function_arn_c = CString::new(ctx.function_arn).unwrap().into_raw();
                let client_ctx_c = CString::new(ctx.client_context.as_str()).unwrap().into_raw();
                let ctx_c = Box::new(gg_lambda_context {
                    function_arn: function_arn_c,
                    client_context: client_ctx_c,
                });
                handler(Box::into_raw(ctx_c));
            }
        }
    }

    pub extern "C" fn gg_lambda_handler_read(
        buffer: *mut ::std::os::raw::c_void,
        buffer_size: usize,
        amount_read: *mut usize,
    ) -> gg_error {
        unsafe {
            GG_LAMBDA_HANDLER_READ_BUFFER.with(|b| {
                let mut borrowed = b.borrow().clone();
                // If the vector is empty, don't do anything and notify that we read zero bytes
                if borrowed.is_empty() {
                    amount_read.write(0);
                } else {
                    // Find the index to split the array off at
                    let index = if buffer_size > borrowed.len() {
                        borrowed.len()
                    } else {
                        buffer_size
                    };
                    // borrowed will now contain everything up to index
                    let next = borrowed.split_off(index);
                    println!("gg_lambda_handler_read: writing buffer: {:?}", borrowed);
                    buffer.copy_from_nonoverlapping(
                        borrowed.as_ptr() as *const c_void,
                        borrowed.len(),
                    );
                    amount_read.write(borrowed.len());
                    // replace the refcell with the rest of the vec
                    b.replace(next);
                }
            });
        }
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_lambda_handler_write_response(
        response: *const ::std::os::raw::c_void,
        response_size: usize,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_lambda_handler_write_error(
        error_message: *const ::std::os::raw::c_char,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct GGGetSecretValueArgs {
        pub ggreq: _gg_request,
        pub secret_id: String,
        pub version_id: Option<String>,
        pub version_stage: Option<String>,
    }

    pub extern "C" fn gg_get_secret_value(
        ggreq: gg_request,
        secret_id: *const ::std::os::raw::c_char,
        version_id: *const ::std::os::raw::c_char,
        version_stage: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_GET_SECRET_VALUE_ARGS.with(|rc| {
                let rust_version_id = if version_id.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(version_id).to_owned().into_string().unwrap())
                };

                let rust_version_stage = if version_stage.is_null() {
                    None
                } else {
                    Some(
                        CStr::from_ptr(version_stage)
                            .to_owned()
                            .into_string()
                            .unwrap(),
                    )
                };
                let args = GGGetSecretValueArgs {
                    ggreq: ggreq.as_ref().unwrap().clone(),
                    secret_id: CStr::from_ptr(secret_id).to_owned().into_string().unwrap(),
                    version_id: rust_version_id,
                    version_stage: rust_version_stage,
                };
                rc.replace(args);
            });
        }
        GG_GET_SECRET_VALUE_RETURN.with(|rc| *rc.borrow())
    }

    pub const gg_invoke_type_GG_INVOKE_EVENT: gg_invoke_type = 0;
    pub const gg_invoke_type_GG_INVOKE_REQUEST_RESPONSE: gg_invoke_type = 1;
    pub const gg_invoke_type_GG_INVOKE_RESERVED_MAX: gg_invoke_type = 2;
    pub const gg_invoke_type_GG_INVOKE_RESERVED_PAD: gg_invoke_type = 2147483647;
    pub type gg_invoke_type = u32;
    pub const gg_runtime_opt_GG_RT_OPT_ASYNC: gg_runtime_opt = 1;
    pub const gg_runtime_opt_GG_RT_OPT_RESERVED_PAD: gg_runtime_opt = 2147483647;
    pub type gg_runtime_opt = u32;

    #[derive(Debug, Copy, Clone)]
    pub struct gg_invoke_options {
        pub function_arn: *const ::std::os::raw::c_char,
        pub customer_context: *const ::std::os::raw::c_char,
        pub qualifier: *const ::std::os::raw::c_char,
        pub type_: gg_invoke_type,
        pub payload: *const ::std::os::raw::c_void,
        pub payload_size: usize,
    }

    #[derive(Debug, Default)]
    pub(crate) struct GGInvokeArgs {
        pub(crate) function_arn: String,
        pub(crate) customer_context: Vec<u8>,
        pub(crate) qualifier: String,
        pub(crate) invoke_type: InvokeType,
        pub(crate) payload: Vec<u8>,
    }

    pub extern "C" fn gg_invoke(
        ggreq: gg_request,
        opts: *const gg_invoke_options,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_INVOKE_ARGS.with(|rc| {
                let mut dst = Vec::with_capacity((*opts).payload_size);
                dst.set_len((*opts).payload_size);
                std::ptr::copy(
                    (*opts).payload as *const u8,
                    dst.as_mut_ptr(),
                    (*opts).payload_size,
                );

                let args = GGInvokeArgs {
                    function_arn: CStr::from_ptr((*opts).function_arn)
                        .to_owned()
                        .into_string()
                        .unwrap(),
                    customer_context: base64::decode(
                        CStr::from_ptr((*opts).customer_context)
                            .to_owned()
                            .into_bytes(),
                    )
                    .unwrap(),
                    qualifier: CStr::from_ptr((*opts).qualifier)
                        .to_owned()
                        .into_string()
                        .unwrap(),
                    invoke_type: InvokeType::try_from((*opts).type_).unwrap(),
                    payload: dst,
                };
                rc.replace(args);
            });
        }

        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_publish_options_init(opts: *mut gg_publish_options) -> gg_error {
        GG_PUBLISH_OPTION_INIT_COUNT.with(|rc| {
            let new_value = *rc.borrow() + 1;
            rc.replace(new_value);
        });
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_publish_options_free(opts: gg_publish_options) -> gg_error {
        GG_PUBLISH_OPTION_FREE_COUNT.with(|rc| {
            let new_value = *rc.borrow() + 1;
            rc.replace(new_value);
        });
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_publish_options_set_queue_full_policy(
        opts: gg_publish_options,
        policy: gg_queue_full_policy_options,
    ) -> gg_error {
        GG_PUBLISH_OPTIONS_SET_QUEUE_FULL_POLICY.with(|rc| {
            rc.replace(policy);
        });
        gg_error_GGE_SUCCESS
    }

    /// Represents arguments passed to gg_publish
    #[derive(Debug, Default, PartialEq)]
    pub struct GGPublishPayloadArgs {
        pub topic: String,
        pub payload: Vec<u8>,
        pub payload_size: usize,
    }

    pub extern "C" fn gg_publish_with_options(
        ggreq: gg_request,
        topic: *const ::std::os::raw::c_char,
        payload: *const ::std::os::raw::c_void,
        payload_size: usize,
        opts: gg_publish_options,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_PUBLISH_WITH_OPTIONS_ARGS.with(|args| {
                // read the void* payload pointer into a byte array
                let mut dst = Vec::with_capacity(payload_size);
                dst.set_len(payload_size);
                std::ptr::copy(payload as *const u8, dst.as_mut_ptr(), payload_size);

                let gg_args = GGPublishPayloadArgs {
                    topic: CStr::from_ptr(topic).to_owned().into_string().unwrap(),
                    payload: dst,
                    payload_size,
                };

                args.replace(gg_args);
            });
        }
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_publish(
        ggreq: gg_request,
        topic: *const ::std::os::raw::c_char,
        payload: *const ::std::os::raw::c_void,
        payload_size: usize,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_PUBLISH_ARGS.with(|args| {
                // read the void* payload pointer into a byte array
                let mut dst = Vec::with_capacity(payload_size);
                dst.set_len(payload_size);
                std::ptr::copy(payload as *const u8, dst.as_mut_ptr(), payload_size);

                let gg_args = GGPublishPayloadArgs {
                    topic: CStr::from_ptr(topic).to_owned().into_string().unwrap(),
                    payload: dst,
                    payload_size,
                };

                args.replace(gg_args);
            });
        }
        gg_error_GGE_SUCCESS
    }

    //noinspection DuplicatedCode
    pub extern "C" fn gg_get_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_SHADOW_THING_ARG.with(|rc| {
                let thing_name_rust = CStr::from_ptr(thing_name).to_owned().into_string().unwrap();
                rc.replace(thing_name_rust);
            });
        }
        gg_error_GGE_SUCCESS
    }

    pub extern "C" fn gg_update_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        update_payload: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_UPDATE_PAYLOAD.with(|rc| {
                let payload = CStr::from_ptr(update_payload)
                    .to_owned()
                    .into_string()
                    .unwrap();
                rc.replace(payload);
            });
            GG_SHADOW_THING_ARG.with(|rc| {
                let thing_name_rust = CStr::from_ptr(thing_name).to_owned().into_string().unwrap();
                rc.replace(thing_name_rust);
            });
        }
        gg_error_GGE_SUCCESS
    }

    //noinspection DuplicatedCode
    pub extern "C" fn gg_delete_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        _result: *mut gg_request_result,
    ) -> gg_error {
        unsafe {
            GG_SHADOW_THING_ARG.with(|rc| {
                let thing_name_rust = CStr::from_ptr(thing_name).to_owned().into_string().unwrap();
                rc.replace(thing_name_rust);
            });
        }
        gg_error_GGE_SUCCESS
    }
}

#[cfg(all(test, not(feature = "coverage")))]
mod bindings_test {
    // This is to make sure binding tests are still run
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
