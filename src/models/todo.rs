use anyhow::Result;
use async_trait::async_trait;
use bson::doc;
use chrono::{DateTime, Utc};
use mongodb::{options::IndexOptions, results::CreateIndexesResult, IndexModel};
use serde::{Deserialize, Serialize};

use aws_rust::database::Model;

#[derive(Debug, Deserialize, Serialize)]
pub struct Todo {
    #[serde(rename = "_id")]
    pub id: String,
    pub task: String,
    pub complete: bool,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Normalized {
    #[serde(rename = "_id")]
    pub id: String,
    pub task: String,
    pub complete: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Todo {
    pub fn normalize(&self) -> Normalized {
        Normalized {
            id: self.id.clone(),
            task: self.task.clone(),
            complete: self.complete,
            created_at: self.created_at.to_string(),
            updated_at: self.updated_at.to_string(),
        }
    }
}

#[async_trait]
impl Model for Todo {
    fn collection_name<'a>() -> &'a str {
        "todos"
    }

    async fn create_indexes() -> Result<Option<CreateIndexesResult>> {
        let complete_index = IndexModel::builder()
            .keys(doc! { "complete": 1 })
            .options(None)
            .build();
        let task_index = IndexModel::builder()
            .keys(doc! { "task": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let indexes = [complete_index, task_index];
        let result = Self::collection()
            .await
            .create_indexes(indexes, None)
            .await?;
        Ok(Some(result))
    }
}
