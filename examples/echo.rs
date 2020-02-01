use aws_greengrass_core_rust::client::IOTDataClient;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::Initializer;
use log::{error, info, LevelFilter};
use std::{thread, time};

const SEND_TOPIC: &'static str = "gg_echo_lambda/device-sent";

struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(&self, ctx: LambdaContext) {
        info!("Handler received: {:?}", ctx);
        let message = String::from_utf8_lossy(ctx.message.as_slice());
        info!("Message: {}", message);
        if let Err(e) = IOTDataClient::publish(SEND_TOPIC, ctx.message.clone()) {
            error!("Error sending {} to topic {} -- {}", message, SEND_TOPIC, e);
        }
    }
}

fn main() {
    gglog::init_log(LevelFilter::Debug);
    info!("Starting gg_echo_handler");

    let runtime = Runtime::default().with_handler(Some(Box::new(EchoHandler)));

    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("green grass initialization error: {}", e)
    } else {
        loop {
            // every two minutes print out a log entry
            let ten_millis = time::Duration::from_secs(60 * 2);
            thread::sleep(ten_millis);
            info!("gg_echo_handler still running");
        }
    }
}
