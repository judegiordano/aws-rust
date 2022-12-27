use anyhow::Result;
use aws_rust::types::{Message, ResponseHelper};
use lambda_http::{run, service_fn, Error, Request, Response};

async fn dev_ping(_: Request) -> Result<Response<String>> {
    let msg = Message {
        message: "sup".to_string(),
    };
    let response = msg.to_response()?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(dev_ping)).await
}
