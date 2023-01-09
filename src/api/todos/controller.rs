use actix_web::{web, HttpResponse};
use bson::doc;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::{todo::Todo, user::User};
use aws_rust::database::{generate_nanoid, ListQueryOptions, Model};

#[derive(Deserialize, Serialize)]
pub struct CreateTodo {
    pub task: String,
    pub user: String,
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
        Ok(inserted) => {
            // save to user
            let update_user = User::update_one(
                doc! { "_id": body.user.to_string() },
                doc! { "$set": { "updated_at": now }, "$push": { "todos": inserted.id.to_string() } }
                ,
            )
            .await;
            if let Err(err) = update_user {
                return HttpResponse::InternalServerError()
                    .json(json!({ "error": err.to_string() }));
            }
            HttpResponse::Created().json(inserted.normalize())
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn complete_todo(query: web::Query<FilterById>) -> HttpResponse {
    match Todo::update_one(
        doc! { "_id": query.id.to_string() },
        doc! { "$set": { "complete": true, "updated_at": chrono::Utc::now() } },
    )
    .await
    {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn list_todos() -> HttpResponse {
    let opts = ListQueryOptions {
        sort: Some(doc! { "complete": 1, "created_at": -1 }),
        ..Default::default()
    };
    match Todo::list(None, Some(opts)).await {
        Ok(found) => {
            let found = found.par_iter().map(Todo::normalize).collect::<Vec<_>>();
            HttpResponse::Ok().json(found)
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn read_todo(path: web::Path<String>) -> HttpResponse {
    let query = Todo::read(Some(doc! { "_id": path.to_owned() }), None).await;
    match query {
        Ok(doc) => doc.map_or_else(
            || HttpResponse::NotFound().json(json!({ "error": "no todo found" })),
            |found| HttpResponse::Ok().json(found.normalize()),
        ),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}
