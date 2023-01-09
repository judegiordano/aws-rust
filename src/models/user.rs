use anyhow::Result;
use async_trait::async_trait;
use bson::doc;
use chrono::{DateTime, Utc};
use mongodb::{options::IndexOptions, results::CreateIndexesResult, IndexModel};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use super::todo::{Normalized as NormalizedTodo, Todo};
use aws_rust::database::Model;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub email: String,
    pub todos: Vec<String>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Populated {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub email: String,
    pub todos: Vec<Todo>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl Populated {
    pub fn normalize(&self) -> Normalized {
        let todos = self
            .todos
            .par_iter()
            .map(super::todo::Todo::normalize)
            .collect::<Vec<_>>();
        Normalized {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            todos,
            created_at: self.created_at.to_string(),
            updated_at: self.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Normalized {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub email: String,
    pub todos: Vec<NormalizedTodo>,
    pub created_at: String,
    pub updated_at: String,
}

#[async_trait]
impl Model for User {
    fn collection_name<'a>() -> &'a str {
        "users"
    }

    async fn create_indexes() -> Result<Option<CreateIndexesResult>> {
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let todos_index = IndexModel::builder()
            .keys(doc! { "todos": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let indexes = [username_index, email_index, todos_index];
        let result = Self::collection()
            .await
            .create_indexes(indexes, None)
            .await?;
        Ok(Some(result))
    }
}
