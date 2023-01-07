use actix_web::{web, HttpResponse};
use bson::doc;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::todo::Todo;
use aws_rust::database::{generate_nanoid, Model};

#[derive(Deserialize, Serialize)]
pub struct CreateTodo {
    pub task: String,
}

#[derive(Deserialize, Serialize)]
pub struct FilterById {
    pub id: String,
}

pub async fn create_todo(body: web::Json<CreateTodo>) -> HttpResponse {
    let now = chrono::Utc::now();
    let todo = Todo {
        id: generate_nanoid(),
        task: body.task.trim().to_owned(),
        complete: false,
        created_at: now,
        updated_at: now,
    };
    match todo.save().await {
        Ok(inserted) => HttpResponse::Ok().json(inserted.normalize()),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn complete_todo(query: web::Query<FilterById>) -> HttpResponse {
    match Todo::update(
        doc! { "_id": query.id.to_string() },
        doc! { "complete": true },
    )
    .await
    {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn list_todos() -> HttpResponse {
    match Todo::list().await {
        Ok(found) => {
            let found = found.par_iter().map(Todo::normalize).collect::<Vec<_>>();
            HttpResponse::Ok().json(found)
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn read_todo(path: web::Path<String>) -> HttpResponse {
    match Todo::read_by_id(&path).await {
        Ok(found) => HttpResponse::Ok().json(found.normalize()),
        Err(err) => HttpResponse::NotFound().json(json!({ "error": err.to_string() })),
    }
}
