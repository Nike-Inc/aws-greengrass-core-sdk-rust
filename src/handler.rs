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
#[derive(Debug)]
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
