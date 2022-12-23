use aws_rust::types::Message;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

async fn ping(event: LambdaEvent<Value>) -> Result<Message, Error> {
    let (_, _) = event.into_parts();
    let msg = Message {
        message: "ping".to_string(),
    };
    Ok(msg)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(ping)).await
}
