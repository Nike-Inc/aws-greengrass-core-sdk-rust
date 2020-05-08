/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

//! Provices an idiomatic Rust API on top of the AWS GreenGrass Core C SDK
//!
//! # Quick Start
//! ```rust
//! use aws_greengrass_core_rust::Initializer;
//! use aws_greengrass_core_rust::log as gglog;
//! use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
//! use log::{info, error, LevelFilter};
//! use aws_greengrass_core_rust::runtime::Runtime;
//!
//! struct HelloHandler;
//!
//! impl Handler for HelloHandler {
//!     fn handle(&self, ctx: LambdaContext) {
//!         info!("Received context: {:#?}", ctx);
//!         let msg = String::from_utf8(ctx.message).expect("Message was not a valid utf8 string");
//!         info!("Received event: {}", msg);
//!     }
//! }
//!
//! gglog::init_log(LevelFilter::Info);
//! let runtime = Runtime::default().with_handler(Some(Box::new(HelloHandler)));
//! if let Err(e) = Initializer::default().with_runtime(runtime).init() {
//!     error!("Initialization failed: {}", e);
//!     std::process::exit(1);
//! }
//! ```
#![allow(unused_unsafe)] // because the test bindings will complain otherwise

mod bindings;
pub mod error;
pub mod handler;
pub mod iotdata;
pub mod lambda;
pub mod log;
pub mod request;
pub mod runtime;
pub mod secret;
pub mod shadow;

use crate::bindings::gg_global_init;
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
        Initializer { runtime }
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

#[cfg(test)]
pub mod test {
    use std::cell::{Ref, RefCell};

    /// Provides a mechanism that can be used to save calls from a Mock implementation
    /// ```rust
    /// use aws_greengrass_core_rust::test::CallHolder;
    /// use std::rc::Rc;
    ///
    /// trait MyTrait {
    ///     fn call(&self, foo: &str);
    /// }
    ///
    /// struct MockImpl {
    ///     call_holder: Rc<CallHolder<String>>
    /// }
    ///
    /// impl MockTrait for MockImpl {
    ///     fn call(&self, foo: &str) {
    ///         self.call_holder.push(foo.to_owned());
    ///     }
    /// }
    /// ```
    pub struct CallHolder<T> {
        calls: RefCell<Vec<T>>,
    }

    impl<T> CallHolder<T> {
        pub fn new() -> Self {
            CallHolder {
                calls: RefCell::new(Vec::<T>::new()),
            }
        }

        /// Push new call information to the internal RefCell
        pub fn push(&self, call: T) {
            self.calls.borrow_mut().push(call)
        }

        /// Return all the calls made
        pub fn calls(&self) -> Ref<Vec<T>> {
            self.calls.borrow()
        }
    }
}
