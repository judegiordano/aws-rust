use actix_web::{web::scope, App, HttpServer};
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};
use tracing_subscriber::FmtSubscriber;

use crate::{
    api,
    models::{todo::Todo, user::User},
};
use aws_rust::{
    config::{self, Env},
    database::Model,
};

pub async fn run() -> anyhow::Result<(), lambda_http::Error> {
    let Env { log_level, .. } = config::Env::default();
    {
        // migrate indexes
        Todo::create_indexes().await?;
        User::create_indexes().await?;
    }
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    // launch
    let factory = move || App::new().service(scope("/api").configure(api::routes));
    if is_running_on_lambda() {
        run_actix_on_lambda(factory).await?;
    } else {
        HttpServer::new(factory)
            .bind(("0.0.0.0", 3000))?
            .run()
            .await?;
    }
    Ok(())
}
