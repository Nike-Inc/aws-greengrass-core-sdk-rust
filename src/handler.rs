//! Provided the handler implementation that can be registered to receive MQTT events
//!
//! # Examples
//!
//! ## Registering a Handler
//! ```rust
//! use aws_greengrass_core_rust::handler::{Handler, LambdaContext, HandlerResult};
//! use aws_greengrass_core_rust::runtime::Runtime;
//! use aws_greengrass_core_rust::Initializer;
//! struct MyHandler;
//! impl Handler for MyHandler {
//!     fn handle(&self, event: Vec<u8>, ctx: LambdaContext) -> HandlerResult {
//!         println!("Received an event! {:?}", ctx);
//!         Ok(None)
//!     }
//! }
//!
//! let runtime = Runtime::default().with_handler(Some(Box::new(MyHandler)));
//! Initializer::default().with_runtime(runtime).init();
//! ```

use std::error::Error;
use std::fmt;

pub type HandlerResult = Result<Option<Vec<u8>>, HandlerError>;

/// Trait to implement for specifying a handler to the greengrass runtime.
/// This provides the ability to listen to messages sent on MQTT topics.alloc
///
/// See [`aws_greengrass_core_rust::runtime::Runtime`] on registering handlers.
pub trait Handler {
    /// Called when a message is sent to this lambda function.
    /// Optionally returns a payload that should be sent to the caller
    ///
    /// # Arguments
    /// * `event` - The event in bytes
    /// * `ctx` - lambda context information
    fn handle(&self, event: Vec<u8>, ctx: LambdaContext) -> HandlerResult;
}


/// Provides information around the the event that was received
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaContext {
    /// The full lambda ARN
    pub function_arn: String,
    /// Client context information
    pub client_context: String,
}

impl LambdaContext {
    pub fn new(function_arn: String, client_context: String) -> Self {
        LambdaContext {
            function_arn,
            client_context,
        }
    }
}

/// Specifies the kind of errors that can happen when an error happens within a lambda. We respond back to greengrass
/// with specified error message
#[derive(Debug)]
pub struct HandlerError(pub String);

impl Error for HandlerError {
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let HandlerError(msg) = &self;
        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let function_arn = "sdlkfjds";
        let client_context = "asdfdfsafdsa";
        let ctx = LambdaContext::new(function_arn.to_owned(), client_context.to_owned());
        assert_eq!(&ctx.function_arn, function_arn);
        assert_eq!(&ctx.client_context, client_context);
    }

}