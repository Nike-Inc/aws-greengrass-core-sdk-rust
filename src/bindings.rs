#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
pub use self::test::*;

/// Provides stubbed testing versions of methods, etc that match greengrasssdk.h
/// Useful for internal testing.
#[cfg(test)]
pub mod test {
    use std::thread_local;
    use std::cell::RefCell;
    use std::os::raw::{c_void, c_char};
    use std::ffi::{CStr, CString};

    #[derive(Debug, Copy, Clone)]
    pub struct _gg_request {
        _unused: [u8; 0],
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

    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_BEST_EFFORT: gg_queue_full_policy_options = 0;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_ALL_OR_ERROR: gg_queue_full_policy_options = 1;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_RESERVED_MAX: gg_queue_full_policy_options = 2;
    pub const gg_queue_full_policy_options_GG_QUEUE_FULL_POLICY_RESERVED_PAD: gg_queue_full_policy_options = 2147483647;

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


    pub fn gg_global_init(opt: u32) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_log(level: gg_log_level, format: *const ::std::os::raw::c_char) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_request_init(ggreq: *mut gg_request) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_request_close(ggreq: gg_request) -> gg_error {
        gg_error_GGE_SUCCESS
    }


    thread_local! {
        pub static GG_REQUEST_READ_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![]);
    }

    pub fn gg_request_read(
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
                    buffer.copy_from_nonoverlapping(borrowed.as_ptr() as *const c_void, borrowed.len());
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
    pub type gg_lambda_handler = ::std::option::Option<unsafe fn(cxt: *const gg_lambda_context)>;

    pub fn gg_runtime_start(handler: gg_lambda_handler, opt: u32) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_lambda_handler_read(
        buffer: *mut ::std::os::raw::c_void,
        buffer_size: usize,
        amount_read: *mut usize,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_lambda_handler_write_response(
        response: *const ::std::os::raw::c_void,
        response_size: usize,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_lambda_handler_write_error(error_message: *const ::std::os::raw::c_char) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_get_secret_value(
        ggreq: gg_request,
        secret_id: *const ::std::os::raw::c_char,
        version_id: *const ::std::os::raw::c_char,
        version_stage: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
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

    pub fn gg_invoke(
        ggreq: gg_request,
        opts: *const gg_invoke_options,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_publish_options_init(opts: *mut gg_publish_options) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_publish_options_free(opts: gg_publish_options) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_publish_options_set_queue_full_policy(
        opts: gg_publish_options,
        policy: gg_queue_full_policy_options,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_publish_with_options(
        ggreq: gg_request,
        topic: *const ::std::os::raw::c_char,
        payload: *const ::std::os::raw::c_void,
        payload_size: usize,
        opts: gg_publish_options,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    /// Represents arguments passed to gg_publish
    #[derive(Debug, Default, PartialEq)]
    pub struct GGPublishPayloadArgs {
        pub topic: String,
        pub payload: Vec<u8>,
        pub payload_size: usize
    }

    thread_local! {
        /// used to store the arguments passed to gg_publish
        pub static GG_PUBLISH_ARGS: RefCell<GGPublishPayloadArgs> = RefCell::new(GGPublishPayloadArgs::default());
    }

    pub fn gg_publish(
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

    pub fn gg_get_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_update_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        update_payload: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }

    pub fn gg_delete_thing_shadow(
        ggreq: gg_request,
        thing_name: *const ::std::os::raw::c_char,
        result: *mut gg_request_result,
    ) -> gg_error {
        gg_error_GGE_SUCCESS
    }
}
