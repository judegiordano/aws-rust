use anyhow::Result;
use async_once::AsyncOnce;
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Deserialize;

pub mod user_model;

use crate::prisma::{self, PrismaClient};

lazy_static! {
    #[derive(Debug, Clone, Copy)]
    pub static ref PRISMA_CLIENT: AsyncOnce<PrismaClient> = AsyncOnce::new(async {
        tracing::info!("connecting to mysql...");
        match prisma::new_client().await {
            Ok(client) => client,
            Err(err) => {
                tracing::error!("connecting to mysql db: {err}");
                std::process::exit(1)
            }
        }
    });
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[async_trait]
pub trait PrismaHelpers<T> {
    async fn paginate(options: PaginationQuery) -> Result<Vec<T>>;
    async fn read_by_id(id: &str) -> Result<Option<T>>;
}
