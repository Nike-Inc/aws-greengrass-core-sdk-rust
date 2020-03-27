//! This is a simple example that will just send a message to an MQTT topic when it is run.
//!
//! This should be deployed in conjunction with the invoker example lambda
use aws_greengrass_core_rust::Initializer;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use log::{info, error, LevelFilter};
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::lambda::LambdaClient;

struct InvokeeHandler;

impl Handler for InvokeeHandler {
    fn handle(&self, ctx: LambdaContext) {
        info!("Received context: {:?}", ctx);
        match String::from_utf8(ctx.message) {
            Ok(msg) => {
                info!("Received event: {}", msg);
                let reply = format!("{{\"original_msg\": \"{}\" }}", msg);
                if let Err(e) = LambdaClient::default().send_response(Ok(reply.as_bytes())) {
                    error!("Error sending response: {}", e);
                }
            }
            Err(e) => {
                let reply = format!("Could not parse message: {}", e);
                error!("{}", reply);
                if let Err(e) = LambdaClient::default().send_response(Err(&reply)) {
                    error!("Error sending error response: {}", e);
                }
            }
        }
    }
}

pub fn main() {
    gglog::init_log(LevelFilter::Info);
    let runtime = Runtime::default().with_handler(Some(Box::new(InvokeeHandler)));
    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("Initialization failed: {}", e);
        std::process::exit(1);
    }
}
