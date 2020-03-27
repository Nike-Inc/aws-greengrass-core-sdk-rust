//! A Simple On Demand Lambda that registers a handler that listens to one MQTT topic and responds to another
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use aws_greengrass_core_rust::iotdata::IOTDataClient;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::{Runtime, RuntimeOption};
use aws_greengrass_core_rust::Initializer;
use log::{error, info, LevelFilter};

/// The topic this Lambda will publish too.
/// A different topic should be wired for this lambda to listen to.
const SEND_TOPIC: &str = "gg_echo_lambda/device-sent";

struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(&self, ctx: LambdaContext) {
        info!("Handler received: {:?}", ctx);
        let message = String::from_utf8_lossy(ctx.message.as_slice());
        info!("Message: {}", message);
        if let Err(e) = IOTDataClient::default().publish(SEND_TOPIC, ctx.message.clone()) {
            error!("Error sending {} to topic {} -- {}", message, SEND_TOPIC, e);
        }
    }
}

fn main() {
    gglog::init_log(LevelFilter::Debug);
    info!("Starting gg_echo_handler");

    let runtime = Runtime::default()
        // On-demand greengrass lambdas should use RuntimeOption::Sync
        .with_runtime_option(RuntimeOption::Sync)
        .with_handler(Some(Box::new(EchoHandler)));

    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("green grass initialization error: {}", e)
    }
}