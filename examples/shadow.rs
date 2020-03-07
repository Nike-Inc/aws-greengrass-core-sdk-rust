use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::{Initializer, GGResult};
use aws_greengrass_core_rust::shadow::ShadowClient;
use serde_json::Value;

use log::{error, info, LevelFilter};
use std::{thread, time};

const DEFAULT_THING_NAME: &'static str = "foo";

struct ShadowHandler {
    thing_name: String,
}

impl ShadowHandler {
    pub fn new() -> Self {
        let thing_name = std::env::var("THING_NAME")
            .unwrap_or(DEFAULT_THING_NAME.to_owned());
        ShadowHandler {
            thing_name
        }
    }
}

impl Handler for ShadowHandler {
    fn handle(&self, _ctx: LambdaContext) {
        if let Err(e) = do_stuff_with_thing() {
            error!("Error calling shadows api: {}", e);
        }
    }
}

fn main() {
    gglog::init_log(LevelFilter::Debug);
    info!("Starting shadow gg lambda");

    let runtime = Runtime::default().with_handler(Some(Box::new(ShadowHandler::new())));

    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("green grass initialization error: {}", e)
    }
}


fn do_stuff_with_thing() -> GGResult<()> {
    let (thing, response) = ShadowClient::default().get_thing_shadow::<Value>("foo")?;
    info!("response: {:?}", response);
    info!("Shadow Thing: {:#?}", thing);
    Ok(())
}