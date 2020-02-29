//! Provices an idiomatic Rust API on top of the AWS GreenGrass Core C SDK
//!
//! # Quick Start
//! ```rust
//! use log::LevelFilter;
//! use aws_greengrass_core_rust::log as gglog;
//! use aws_greengrass_core_rust::client::IOTDataClient;
//! use aws_greengrass_core_rust::init;
//!
//! pub fn main() -> std::io::Result<()> {
//!     gglog::init_log(LevelFilter::Info);
//!     init().map_err(|e| e.as_ioerror());
//!     let _result = IOTDataClient::default().publish("mytopic", r#"{"msg": "foo"}"#).map_err(|e| e.as_ioerror());
//!     Ok(())
//! }
//! ```
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod client;
pub mod error;
pub mod handler;
pub mod log;
pub mod runtime;
pub mod secret;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use crate::error::GGError;
use crate::runtime::Runtime;
use std::default::Default;

pub type GGResult<T> = Result<T, GGError>;

/// Provides the ability initialize the greengrass runtime
pub struct Initializer {
    runtime: Runtime,
}

impl Initializer {
    pub fn init(self) -> GGResult<()> {
        unsafe {
            // At this time there are no options for gg_global_init
            let init_res = gg_global_init(0);
            GGError::from_code(init_res)?;
            self.runtime.start()?;
        }
        Ok(())
    }

    /// Initialize the greengrass with the specified runtime object.
    ///
    /// This must be called if you want to provide a Runtime with a [`handler::Handler`].
    ///
    /// ```edition2018
    /// use aws_greengrass_core_rust::runtime::Runtime;
    /// use aws_greengrass_core_rust::Initializer;
    ///
    /// Initializer::default().with_runtime(Runtime::default());
    /// ```
    pub fn with_runtime(self, runtime: Runtime) -> Self {
        Initializer { runtime, ..self }
    }
}

/// Creates a Initializer with the default Runtime
impl Default for Initializer {
    fn default() -> Self {
        Initializer {
            runtime: Runtime::default(),
        }
    }
}

/// Initialize the Greengrass runtime without a handler
pub fn init() -> GGResult<()> {
    Initializer::default().init()
}
