use aws_rust::config::{self, Env};
use tracing_subscriber::FmtSubscriber;

pub mod api;
pub mod server;

#[tokio::main]
async fn main() -> anyhow::Result<(), lambda_http::Error> {
    let Env { log_level, .. } = config::Env::default();
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    // launch server
    server::run().await?;
    Ok(())
}
