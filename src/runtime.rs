include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use std::default::Default;

pub enum RuntimeOption {
    Async,
}

impl RuntimeOption {
    fn as_opt(&self) -> gg_runtime_opt {
        match self {
            Async => 1,
        }
    }
}

pub struct Runtime {
    runtime_option: RuntimeOption,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime {
            runtime_option: RuntimeOption::Async,
        }
    }
}

impl Runtime {
    pub fn start(self) -> Result<(), GGError> {
        unsafe {
            // todo - support handlers
            // this will probably involve creating a handler with a channel that then
            // sends info to our handler function: https://doc.rust-lang.org/nomicon/ffi.html#asynchronous-callbacks
            extern "C" fn no_op_handler(_: *const gg_lambda_context) {};
            let start_res = gg_runtime_start(Some(no_op_handler), self.runtime_option.as_opt());
            GGError::from_code(start_res)?;
        }
        Ok(())
    }

    pub fn with_runtime_option(self, runtime_option: RuntimeOption) -> Self {
        Runtime {
            runtime_option,
            ..self
        }
    }
}
