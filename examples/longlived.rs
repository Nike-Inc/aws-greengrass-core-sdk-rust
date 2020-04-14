//! An example of a long-lived green grass lambda.
//! This example creates an http endpoint on the greengrass device. It will then forward
//! messages it receives to the MQTT topic longlived/device-sent.
//!
//! See the following guide for long lived functions: https://docs.aws.amazon.com/greengrass/latest/developerguide/long-lived.html
//!
//! ## Sample Request
//! ```shell script
//! curl -vvvv -H "Content-Type: application/json" -d '{"msg": "hello"}' http://127.0.0.1:5020/
//! ```
use aws_greengrass_core_rust::iotdata::IOTDataClient;
use aws_greengrass_core_rust::log as gglog;
use aws_greengrass_core_rust::runtime::{Runtime, RuntimeOption};
use aws_greengrass_core_rust::{GGResult, Initializer};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::{error, info, LevelFilter};

const SEND_TOPIC: &str = "longlived/device-sent";

async fn serve(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Simply echo the body back to the client.
        (&Method::POST, "/") => {
            let body = hyper::body::to_bytes(req.into_body()).await?;
            match publish(&body).await {
                Ok(_) => {
                    let mut accepted = Response::default();
                    *accepted.status_mut() = StatusCode::ACCEPTED;
                    Ok(accepted)
                }
                Err(e) => {
                    error!("greengrass error occurred: {}", e);
                    let mut internal_error = Response::new(Body::from(format!("{}", e)));
                    *internal_error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(internal_error)
                }
            }
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn publish(bytes: &[u8]) -> GGResult<()> {
    // convert to a string for logging purposes
    info!("publishing message of {}", String::from_utf8_lossy(bytes));
    IOTDataClient::default().publish(SEND_TOPIC, bytes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    gglog::init_log(LevelFilter::Debug);

    // Initialize Greengrass, long lived functions need to be configured with RuntimeOption::Async
    let runtime = Runtime::default().with_runtime_option(RuntimeOption::Async);
    Initializer::default().with_runtime(runtime).init()?;

    // Initialize hyper
    let addr = ([0, 0, 0, 0], 5020).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(serve)) });
    let server = Server::bind(&addr).serve(service);
    info!("Listening on http://{}", addr);
    server.await?;
    info!("longlived lambda exiting");
    Ok(())
}
