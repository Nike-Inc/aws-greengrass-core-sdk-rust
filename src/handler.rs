/// Provided the handler upon receiving an event on an MQTT topic
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
    pub(crate) fn new(function_arn: String, client_context: String, message: Vec<u8>) -> Self {
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
