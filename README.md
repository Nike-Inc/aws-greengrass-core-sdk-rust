# aws-greengrass-core-sdk-rust
Provides an idiomatic Rust API around the [AWS Greengrass Core C SDK](https://github.com/aws/aws-greengrass-core-sdk-c) to mroe easily enable Greengrass native lambda functions in Rust.

## Currently Supported
* Publishing to MQTT topics
* Registering handlers and receiving messages from MQTT topics
* Logging to the greengrass logging backend via the log crate
* Acquiring Secrets

## Examples
* [hello.rs](https://github.nike.com/SensorsPlatform/aws-greengrass-core-sdk-rust/blob/master/examples/hello.rs) - Simple example for initializing the greengrass runtime and sending a message on a topic
* [echo.rs](https://github.nike.com/SensorsPlatform/aws-greengrass-core-sdk-rust/blob/master/examples/echo.rs) - Example that shows how to register a Handler with the greengrass runtime and listen for message.
