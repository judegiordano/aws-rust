use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::{
    prisma::user,
    prisma_models::{user_model::CreateUser, PaginationQuery, PrismaHelpers},
};

pub async fn list_users(query: web::Query<PaginationQuery>) -> HttpResponse {
    let users = user::Data::paginate(query.into_inner()).await;
    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn read_by_id(id: web::Path<String>) -> HttpResponse {
    let user = user::Data::read_by_id(&id).await;
    match user {
        Ok(user) => user.map_or_else(
            || HttpResponse::NotFound().json(json!({ "error": "no user found" })),
            |found| HttpResponse::Ok().json(found),
        ),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn create_user(body: web::Json<CreateUser>) -> HttpResponse {
    let created = user::Data::create(body.into_inner()).await;
    match created {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}

pub async fn delete_by_id(id: web::Path<String>) -> HttpResponse {
    let user = user::Data::delete(&id).await;
    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().json(json!({ "error": err.to_string() })),
    }
}
