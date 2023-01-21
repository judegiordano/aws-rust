use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::prisma::{user, PrismaClient};

pub async fn list_users(client: web::Data<PrismaClient>) -> HttpResponse {
    let users = client
        .user()
        .find_many(vec![])
        .with(user::addresses::fetch(vec![]))
        .exec()
        .await;
    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn read_by_id(client: web::Data<PrismaClient>, id: web::Path<String>) -> HttpResponse {
    let user = client
        .user()
        .find_first(vec![user::id::equals(id.to_string())])
        .with(user::addresses::fetch(vec![]))
        .exec()
        .await;
    match user {
        Ok(user) => user.map_or_else(
            || HttpResponse::NotFound().json(json!({ "error": "no user found" })),
            |found| HttpResponse::Ok().json(found),
        ),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}
