# aws-greengrass-core-sdk-rust
Provides an idiomatic Rust API around the [AWS Greengrass Core C SDK](https://github.com/aws/aws-greengrass-core-sdk-c) to mroe easily enable Greengrass native lambda functions in Rust.

## Supporting functionality
* Publishing to MQTT topics
* Registering handlers and receiving messages from MQTT topics
* Logging to the greengrass logging backend via the log crate
* Acquiring Secrets

## Testing

When the feature "mock" is turned during the test phase the various clients will:

1. Allow you outputs to be overridden
2. Save arguments that methods have been called with

### Example
```rust
  #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_publish_str() {
            let topic = "foo";
            let message = "this is my message";

            let mocks = MockHolder::default().with_publish_raw_outputs(vec![Ok(())]);
            let client = IOTDataClient::default().with_mocks(mocks);
            let response = client.publish(topic, message).unwrap();
            println!("response: {:?}", response);

            let PublishRawInput(raw_topic, raw_bytes, raw_read) =
                &client.mocks.publish_raw_inputs.borrow()[0];
            assert_eq!(raw_topic, topic);
        }
    }
```   

## Examples
* [hello.rs](https://github.nike.com/SensorsPlatform/aws-greengrass-core-sdk-rust/blob/master/examples/hello.rs) - Simple example for initializing the greengrass runtime and sending a message on a topic
* [echo.rs](https://github.nike.com/SensorsPlatform/aws-greengrass-core-sdk-rust/blob/master/examples/echo.rs) - Example that shows how to register a Handler with the greengrass runtime and listen for message.
* [shadow.rs](https://github.nike.com/SensorsPlatform/aws-greengrass-core-sdk-rust/blob/master/examples/shadow.rs) - Example showing how to acquire and manipulate shadow documents.
