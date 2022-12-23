use aws_rust::types::Message;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

async fn pong(event: LambdaEvent<Value>) -> Result<Message, Error> {
    let (_, _) = event.into_parts();
    let msg = Message {
        message: "pong".to_string(),
    };
    Ok(msg)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(pong)).await
}
