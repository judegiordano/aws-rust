use anyhow::Result;
use aws_rust::types::{Message, ResponseHelper};
use lambda_http::{run, service_fn, Error, Request, Response};

async fn pong(_: Request) -> Result<Response<String>> {
    let msg = Message {
        message: "pong".to_string(),
    };
    let response = msg.to_response()?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(pong)).await
}
