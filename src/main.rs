pub mod api;
pub mod models;
pub mod prisma;
pub mod server;

#[tokio::main]
async fn main() -> anyhow::Result<(), lambda_http::Error> {
    server::run().await
}
