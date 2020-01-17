#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod client;
pub mod handler;
pub mod error;
pub mod log;
pub mod runtime;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use crate::runtime::Runtime;
use std::default::Default;

pub struct Initializer {
    runtime: Runtime,
}

impl Initializer {
    pub fn init(self) -> Result<(), GGError> {
        unsafe {
            let init_res = gg_global_init(0);
            GGError::from_code(init_res)?;
        }
        Ok(())
    }

    pub fn with_runtime(self, runtime: Runtime) -> Self {
        Initializer { runtime, ..self }
    }
}

impl Default for Initializer {
    fn default() -> Self {
        Initializer {
            runtime: Runtime::default(),
        }
    }
}

pub fn init() -> Result<(), GGError> {
    Initializer::default().init()
}
