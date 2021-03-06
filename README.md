# AWS Greengrass Core Rust SDK
Provides an idiomatic Rust wrapper around the [AWS Greengrass Core C SDK](https://github.com/aws/aws-greengrass-core-sdk-c) to more easily enable Greengrass native lambda functions in Rust.

## Features
* Publishing to MQTT topics
* Registering handlers and receiving messages from MQTT topics
* Logging to the Greengrass logging backend via the log crate
* Acquiring Secrets

## Examples
* [hello.rs](./examples/hello.rs) - Simple example for initializing the greengrass runtime and sending a message on a topic
* [echo.rs](./examples/echo.rs) - Example that shows how to register a Handler with the greengrass runtime and listen for message.
* [shadow.rs](./examples/shadow.rs) - Example showing how to acquire and manipulate shadow documents.
* [longlived.rs](./examples/longlived.rs) - Example showing how to create a longlived greengrass lambda that exposes a http endpoint.
* [invoker.rs](./examples/invoker.rs) - An example of invoking one lambda for another lambda. Should be used with [invokee.rs](./examples/invokee.rs)

### Building examples
Examples can be built following the directions in Quick start. Use ```cargo build --example <example>``` to build. 

## Quickstart

### Prerequisites and Requirements

* Install the [Greengrass C SDK](https://github.com/aws/aws-greengrass-core-sdk-c) (fails on Mac OS X, see note below)
* Install [Rust](https://www.rust-lang.org/)
* Install the [AWS CLI](https://aws.amazon.com/cli/)
* A Device running green grass version v1.6 or newer
* Create and configure a Greengrass group as described in the [Getting started with Amazon Greengrass](https://docs.aws.amazon.com/greengrass/latest/developerguide/gg-gs.html)

#### Note for Building on mac
The C Greengrass SDK fails to build on Mac OS X. The stubs directory contains a simple stubbed version of the SDK that 
can be used for compiling against Mac OS X.

To Install:
1. ```cd stubs```
2. ```mkdir build && cd build```
3. ```cmake ..```
4. ```make```
5. ```make install``` 

### Create new cargo project

```cargo new --bin my_gg_lambda```

### Add the library to the Cargo.toml
[//]: <> ("Update from git url to version once published to crates.io")
Additionally, defined the logging crate
```toml
aws_greengrass_core_rust = "0.1.36"
log = "^0.4"
```
 
### Edit main.rs 
1. Initialize logging, greengrass runtime, and register a Handler

[//]: <> ("Update from git url to version once published to crates.io")

```rust
use aws_greengrass_core_rust::Initializer;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::handler::{Handler, LambdaContext};
use log::{info, error, LevelFilter};
use aws_greengrass_core_rust::runtime::Runtime;

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
```

### Build and package your lambda function
```shell script
cargo build --release
zip  zip -j my_gg_lambda.zip "./target/release/my_gg_lambda"
```

Note: The binaries must be built on the operating system and architecture you are deploying to. If you are not on linux (Mac OS/windows) you can use the docker build:
```./dockerbuild.sh cargo build```

This will only work for x86 builds. 


### Deploy your lambda function
Using the information you used when creating your Greengrass group:
```shell script
aws lambda create-function \
    --region aws-region \
    --function-name my_gg_lambda_x86 \
    --handler executable-name \
    --role role-arn \
    --zip-file fileb://file-name.zip \
    --runtime arn:aws:greengrass:::runtime/function/executable

aws lambda publish-version \
    --function-name my_gg_lambda_x86 \
    --region aws-region

aws lambda create-alias \
    --function-name my_gg_lambda_x86 \
    --name alias-name \
    --function-version version-number \
    --region aws-region
```
Note: We recommend adding an architecture suffix like x86 or arm to the lambda name if you are planning on deploying to 
multiple architectures.

### Configure your lambda function in your greengrass group
Follow the instructions found in [Configure the Lambda Function for AWS IoT Greengrass](https://docs.aws.amazon.com/greengrass/latest/developerguide/config-lambda.html) 

### Further reading:
* [Run Lambda Functions on the AWS IoT Greengrass Core](https://docs.aws.amazon.com/greengrass/latest/developerguide/lambda-functions.html)


### Testing in your project

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

## Building from source

## Building

1. ```cargo build```

## Testing Mock feature
The examples will not build appropriately when the mock feature is enabled. To run the tests you must skip the examples:
```cargo test --features mock --lib```

## Testing with code coverage

There are some issues with coverage tools running correctly with our bindgen configuration in build.rs. Most of the tests do not
actually need this as bindings.rs contains a mock set of bindings. To get around the failure the feature "coverage" can be enabled.
This will avoid the bindings being generate and disable the couple of spots where the real bindings are needed.

### Coverage with [grcov](https://github.com/mozilla/grcov)
1. Install [gperftools](https://github.com/gperftools/gperftools)
2. Install Rust nightly: ```rustup install nightly```
3. Install grcov: ```cargo +nightly install grcov```
4. Set the following environment variables:
```shell script
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
```  
5. Build with coverage information:
```shell script
cargo clean
cargo +nightly build --features coverage
cargo +nightly test --features coverage
```
6. Run grcov:
```shell script
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/
```

### Cross compilation ###

```shell script
AWS_GREENGRASS_STUBS=yes CMAKE_TOOLCHAIN_FILE=$(pwd)/linux-gnu-x86_64.cmake cargo build --target=x86_64-unknown-linux-gnu
```

### Coverage with [Jetbrains CLion](https://www.jetbrains.com/clion/)
1. Create a run coverage named Test
2. Set the command to be: ```test --features coverage```
3. Run with coverage
