use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DevResponse {
    pub ok: bool,
    pub received: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreatePing {
    pub message: String,
}

pub async fn ping(body: web::Json<CreatePing>) -> HttpResponse {
    HttpResponse::Ok().json(DevResponse {
        ok: true,
        received: body.message.to_string(),
    })
}
