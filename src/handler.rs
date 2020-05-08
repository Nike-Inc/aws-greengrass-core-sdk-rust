/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

//! Provided the handler implementation that can be registered to receive MQTT events
//!
//! # Examples
//!
//! ## Registering a Handler
//! ```rust
//! use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
//! use aws_greengrass_core_rust::runtime::Runtime;
//! use aws_greengrass_core_rust::Initializer;
//! struct MyHandler;
//! impl Handler for MyHandler {
//!     fn handle(&self, ctx: LambdaContext) {
//!         println!("Received an event! {:?}", ctx);
//!     }
//! }
//!
//! let runtime = Runtime::default().with_handler(Some(Box::new(MyHandler)));
//! Initializer::default().with_runtime(runtime).init();
//! ```

/// Provides information around the the event that was received
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaContext {
    /// The full lambda ARN
    pub function_arn: String,
    /// Client context information
    pub client_context: String,
    /// The message received in bytes
    pub message: Vec<u8>,
}

impl LambdaContext {
    pub fn new(function_arn: String, client_context: String, message: Vec<u8>) -> Self {
        LambdaContext {
            function_arn,
            client_context,
            message,
        }
    }
}

/// Trait to implement for specifying a handler to the greengrass runtime.
/// This provides the ability to listen to messages sent on MQTT topics.alloc
///
/// See [`aws_greengrass_core_rust::runtime::Runtime`] on registering handlers.
pub trait Handler {
    fn handle(&self, ctx: LambdaContext);
}

#[cfg(test)]
mod test {
    use crate::handler::LambdaContext;

    #[test]
    fn test_new() {
        let function_arn = "sdlkfjds";
        let client_context = "asdfdfsafdsa";
        let message = "asdjkdsfajl".as_bytes().to_vec();
        let ctx = LambdaContext::new(
            function_arn.to_owned(),
            client_context.to_owned(),
            message.clone(),
        );
        assert_eq!(&ctx.function_arn, function_arn);
        assert_eq!(&ctx.client_context, client_context);
        let cloned = ctx.message.to_owned();
        assert_eq!(cloned, message.clone());
    }
}
