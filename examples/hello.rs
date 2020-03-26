//! This is a simple example that will just send a message to an MQTT topic when it is run.
use aws_greengrass_core_rust::Initializer;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext, HandlerResult, HandlerError};
use log::{info, error, LevelFilter};
use aws_greengrass_core_rust::runtime::Runtime;

struct HelloHandler;

impl Handler for HelloHandler {
    fn handle(&self, event: Vec<u8>, _: LambdaContext) -> HandlerResult {
        let msg = String::from_utf8(event)
            .map_err(|e| HandlerError(format!("{}", e)))?;
        info!("Received: {}", msg);
        let reply = format!("Hello! {}", msg);
        Ok(Some(reply.into_bytes()))
    }
}

pub fn main() {
    gglog::init_log(LevelFilter::Info);
    let runtime = Runtime::default().with_handler(Some(Box::new(HelloHandler)));
    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("Initialization failed: {}", e);
        std::process::exit(1);
    }
}
