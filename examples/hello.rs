/*
 * Copyright 2020-present, Nike, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the Apache-2.0 license found in
 * the LICENSE file in the root of this source tree.
 */

//! This is a simple example that will just send a message to an MQTT topic when it is run.
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::Initializer;
use log::{error, info, LevelFilter};

struct HelloHandler;

impl Handler for HelloHandler {
    fn handle(&self, ctx: LambdaContext) {
        info!("Received context: {:#?}", ctx);
        let msg = String::from_utf8(ctx.message).expect("Message was not a valid utf8 string");
        info!("Received event: {}", msg);
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
