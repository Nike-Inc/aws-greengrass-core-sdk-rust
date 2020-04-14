//! An example of a lambda that takes a response that it receives on a queue and invokes another lambda
//! And returns the response back.
//!
//! It is recommended that the invokee lambda is deployed along with this lambda.
//!
//! This can tested in the AWS IOT test tool by setting up two MQTT subscriptions:
//! * IOT Cloud -> this lambda (i.e. invoker/device-rcvd)
//! * this lambda -> iot cloud (i.e. invoker/device-sent)
//!
//! A request can be sent on from the test tool to the lambda like this (change arn and versions):
//! ```json
//! {
//!     "function_arn": "arn:aws:lambda:us-west-2:701603852992:function:invokee_x86:2",
//!     "qualifier": "2",
//!     "payload": "{\"msg\": \"hello lambda\"}",
//!     "response_topic": "invoker/device-sent"
//! }
//!```
//! Note: Determining the ARN for the greengrass lambda isn't easy in the console. The following command will work
//! ```shell script
//! aws lambda list-versions-by-function --function-name <function name> --output yaml
//! ```
use aws_greengrass_core_rust::error::GGError;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use aws_greengrass_core_rust::iotdata::IOTDataClient;
use aws_greengrass_core_rust::lambda::{InvokeOptions, LambdaClient};
use aws_greengrass_core_rust::log as gg_log;
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::{GGResult, Initializer};
use log::{error, info, LevelFilter};
use serde::Deserialize;
use serde_json::Value;

pub fn main() {
    gg_log::init_log(LevelFilter::Debug);
    let runtime = Runtime::default().with_handler(Some(Box::new(InvokerHandler)));
    if let Err(e) = Initializer::default().with_runtime(runtime).init() {
        error!("Error initializing: {}", e);
        std::process::exit(1);
    }
}

struct InvokerHandler;

impl Handler for InvokerHandler {
    fn handle(&self, ctx: LambdaContext) {
        info!("Received context: {:?}", ctx);
        if let Err(e) = invoke(&ctx.message) {
            error!("An error occurred handling event {}", e);
        }
    }
}

fn invoke(event: &[u8]) -> GGResult<()> {
    let req = InvokeRequest::from_slice(event)?;
    info!("Received event: {:?}", req);
    let options = build_invoke_options(&req)?;
    info!("Attempting to invoke {:?} with {:?}", options, req.payload);
    let resp = LambdaClient::default().invoke_sync(options, Some(req.payload))?;
    if let Some(resp) = resp {
        // convert the payload to a string for logging purposes
        let payload = String::from_utf8(resp).map_err(GGError::from)?;
        info!(
            "Responding to topic: {} with payload {}",
            req.response_topic, payload
        );
        IOTDataClient::default().publish(&req.response_topic, payload)?;
    }
    Ok(())
}

fn build_invoke_options(req: &InvokeRequest) -> GGResult<InvokeOptions<Value>> {
    // not really doing anything interesting with the context, so just put something that parses in it
    let context = serde_json::from_str(r#"{"foo": "bar"}"#).map_err(GGError::from)?;
    let opts = InvokeOptions::new(req.function_arn.clone(), context, req.qualifier.clone());
    Ok(opts)
}

/// Represents the json request that this lambda will respond too
#[derive(Deserialize, Debug)]
struct InvokeRequest {
    function_arn: String,
    qualifier: String,
    payload: String,
    response_topic: String,
}

impl InvokeRequest {
    fn from_slice(slice: &[u8]) -> GGResult<Self> {
        serde_json::from_slice(slice).map_err(GGError::from)
    }
}
