//! An on-demand lambda function that will manipulate a specified thing's shadow document.
//! This lambda will listen on an MQTT topic for a json command body. The following actions can be performed:
//!
//! # Get Shadow Document
//! This will acquire a shadow document for a specified device and send it back to a specified MQTT Topic
//!
//! ## Example Payload:
//! ```json
//! {
//!     "command": "GET",
//!     "thing_name": "myThingName"
//! }```
//!
//! # Delete Shadow Document
//! Delete the shadow document for a specified device
//!
//! ## Example Payload
//! ```json
//! {
//!     "command": "DELETE",
//!     "thing_name": "myThingName"
//! }```
//!
//! # Update Shadow Document
//! Update the shadow document for a specified device
//! ```json
//! {
//!     "command": "UPDATE".
//!     "thing_name": "myThingName",
//!     "document": {
//!         "state" : {
//!             "desired" : {
//!                 "color" : "RED",
//!                 "sequence" : [ "RED", "GREEN", "BLUE" ]
//!             }
//!         }
//!     }
//! }```
use aws_greengrass_core_rust::error::GGError;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext, HandlerResult};
use aws_greengrass_core_rust::iotdata::IOTDataClient;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::Runtime;
use aws_greengrass_core_rust::shadow::ShadowClient;
use aws_greengrass_core_rust::{GGResult, Initializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::default::Default;

use log::{error, info, LevelFilter};

const DEFAULT_SEND_TOPIC: &str = "/shadow-example/device-sent";

struct ShadowHandler {
    iot_data_client: IOTDataClient,
    shadow_client: ShadowClient,
    send_topic: String,
}

impl ShadowHandler {
    pub fn new() -> Self {
        let send_topic = std::env::var("SEND_TOPIC").unwrap_or(DEFAULT_SEND_TOPIC.to_owned());

        ShadowHandler {
            iot_data_client: IOTDataClient::default(),
            shadow_client: ShadowClient::default(),
            send_topic,
        }
    }

    fn do_stuff_with_thing(&self, body: &[u8]) -> GGResult<()> {
        match serde_json::from_slice::<Command>(body) {
            Ok(ref command) => self.handle_command(command),
            Err(_) => {
                let response: Response<EmptyBody> = Response::default()
                    .with_code(400)
                    .with_message(Some("Did not receive a valid command object".to_owned()));
                self.publish(&response)
            }
        }
    }

    fn handle_command(&self, c: &Command) -> GGResult<()> {
        match c.r#type {
            // Get will attempt to grab a document and publish it to an MQTT topic
            CommandType::Get => self.handle_get(c),
            // Attempt to update a shadow thing based on the document we received
            CommandType::Update => self.handle_update(c),
            // Delete the shadow document for the specified thing
            CommandType::Delete => self.handle_delete(c),
        }
    }

    fn handle_get(&self, command: &Command) -> GGResult<()> {
        match self
            .shadow_client
            .get_thing_shadow::<Value>(&command.thing_name)
        {
            // We grabbed the document, send it.
            Ok(Some(thing)) => {
                info!("Shadow Thing: {:#?}", thing);
                let response = Response::default()
                    .with_code(200)
                    .with_body(Some(Box::new(thing)));
                self.publish(&response)
            }
            // We got a 404 back, respond
            Ok(_) => {
                info!("No shadow doc found for thing: {:?}", &command.thing_name);
                let response: Response<EmptyBody> = Response::default()
                    .with_code(404)
                    .with_message(Some("No shadow document for thing".to_owned()));
                self.publish(&response)
            }
            Err(ref e) => self.handle_error(e),
        }
    }

    fn handle_update(&self, command: &Command) -> GGResult<()> {
        if let Some(ref document) = command.document {
            match self
                .shadow_client
                .update_thing_shadow(&command.thing_name, &document)
            {
                Ok(_) => {
                    let response: Response<EmptyBody> = Response::default()
                        .with_code(200)
                        .with_message(Some(format!(
                            "Updated shadow for thing {} successfully",
                            command.thing_name
                        )));
                    self.publish(&response)
                }
                Err(ref e) => self.handle_error(e),
            }
        } else {
            let response: Response<EmptyBody> =
                Response::default()
                    .with_code(400)
                    .with_message(Some(format!(
                        "No document specified to update thing: {}",
                        command.thing_name
                    )));
            self.publish(&response)
        }
    }

    fn handle_delete(&self, command: &Command) -> GGResult<()> {
        match self.shadow_client.delete_thing_shadow(&command.thing_name) {
            Ok(_) => {
                let response: Response<EmptyBody> = Response::default()
                    .with_code(200)
                    .with_message(Some(format!(
                        "Shadow for thing {} successfully deleted.",
                        command.thing_name
                    )));
                self.publish(&response)
            }
            Err(ref e) => self.handle_error(e),
        }
    }

    fn handle_error(&self, err: &GGError) -> GGResult<()> {
        match err {
            GGError::ErrorResponse(e) => {
                let code = e.error_response.as_ref().map(|er| er.code).unwrap_or(500);
                let response = Response::default()
                    .with_code(code)
                    .with_body(Some(Box::new(e.clone())));
                self.publish(&response)
            }
            _ => {
                let response: Response<EmptyBody> = Response::default()
                    .with_code(500)
                    .with_message(Some(format!("Error occurred: {}", err)));
                self.publish(&response)
            }
        }
    }

    fn publish<T: Serialize>(&self, response: &Response<T>) -> GGResult<()> {
        self.iot_data_client
            .publish_json(&self.send_topic, response)
            .map(|_| ())
    }
}

impl Handler for ShadowHandler {
    fn handle(&self, event: Vec<u8>, _: LambdaContext) -> HandlerResult {
        if let Err(e) = self.do_stuff_with_thing(&event) {
            error!("Error calling shadows api: {}", e);
        }
        Ok(None)
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

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CommandType {
    Update,
    Get,
    Delete,
}

#[derive(Deserialize)]
struct Command {
    thing_name: String,
    r#type: CommandType,
    document: Option<Value>,
}

#[derive(Serialize, Default)]
struct Response<T: Serialize> {
    code: u16,
    message: Option<String>,
    body: Option<Box<T>>,
}

impl<T: Serialize> Response<T> {
    fn with_code(self, code: u16) -> Self {
        Response { code, ..self }
    }

    fn with_message(self, message: Option<String>) -> Self {
        Response { message, ..self }
    }

    fn with_body(self, body: Option<Box<T>>) -> Self {
        Response { body, ..self }
    }
}

/// Use to statisfy type constraints when there isn't a body
#[derive(Serialize, Debug, Default)]
struct EmptyBody;
