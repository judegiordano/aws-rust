use actix_web::{web::scope, App, HttpServer};
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};

use crate::api;

pub async fn run() -> anyhow::Result<(), lambda_http::Error> {
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