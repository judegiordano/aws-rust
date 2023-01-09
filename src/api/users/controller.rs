use actix_web::{web, HttpResponse};
use aws_rust::database::{generate_nanoid, Model};
use bson::doc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::user::{Populated, User};

#[derive(Deserialize, Serialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
}

pub async fn create_user(body: web::Json<CreateUser>) -> HttpResponse {
    let now = chrono::Utc::now();
    let user = User {
        id: generate_nanoid(),
        username: body.username.clone(),
        email: body.email.clone(),
        todos: vec![],
        created_at: now,
        updated_at: now,
    };
    match user.save().await {
        Ok(inserted) => HttpResponse::Created().json(inserted),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn read_user(path: web::Path<String>) -> HttpResponse {
    let query = User::read_populate::<Populated>(doc! { "_id": path.to_owned() }, &["todos"]).await;
    match query {
        Ok(doc) => doc.map_or_else(
            || HttpResponse::NotFound().json(json!({ "error": "no user found" })),
            |found| HttpResponse::Ok().json(found.normalize()),
        ),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}
