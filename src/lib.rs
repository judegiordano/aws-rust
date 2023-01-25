pub mod types {
    use anyhow::Result;
    use lambda_http::{http::StatusCode, Response};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::fmt::Debug;

    pub trait ResponseHelper: Serialize {
        fn to_response(&self) -> Result<Response<String>> {
            let body = json!(self).to_string();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(body)?;
            Ok(response)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Message {
        pub message: String,
    }

    impl ResponseHelper for Message {}
}

pub mod config {
    pub struct Env {
        pub log_level: tracing::Level,
        pub mongo_uri: String,
    }

    impl Default for Env {
        fn default() -> Self {
            if cfg!(debug_assertions) {
                use dotenv::dotenv;
                dotenv().ok();
            }
            Self {
                log_level: std::env::var("LOG_LEVEL").map_or(tracing::Level::ERROR, |found| {
                    match found.to_uppercase().as_ref() {
                        "INFO" => tracing::Level::INFO,
                        "DEBUG" => tracing::Level::DEBUG,
                        "WARN" => tracing::Level::WARN,
                        "TRACE" => tracing::Level::TRACE,
                        _ => tracing::Level::ERROR,
                    }
                }),
                mongo_uri: std::env::var("MONGO_URI").map_or(
                    "mongodb://localhost:27017/rust-aws-local".to_string(),
                    |uri| uri,
                ),
            }
        }
    }
}

pub mod database {
    use std::fmt::Debug;

    use anyhow::Result;
    use async_once::AsyncOnce;
    use async_trait::async_trait;
    use bson::{doc, Document};
    use futures::stream::TryStreamExt;
    use lazy_static::lazy_static;
    use mongodb::{
        error::Error as MongoError,
        options::{ClientOptions, FindOneOptions, FindOptions},
        results::{CreateIndexesResult, UpdateResult},
        Client, Collection, Database,
    };
    use nanoid::nanoid;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    use crate::config::Env;

    lazy_static! {
        pub static ref DATABASE: AsyncOnce<Database> = AsyncOnce::new(async {
            let Env { mongo_uri, .. } = Env::default();
            let client_options = ClientOptions::parse(mongo_uri).await.map_or_else(
                |err| {
                    tracing::error!("error parsing client options {err:?}");
                    std::process::exit(1);
                },
                |opts| opts,
            );
            let client = Client::with_options(client_options).map_or_else(
                |err| {
                    tracing::error!("error connecting client: {err:?}");
                    std::process::exit(1);
                },
                |client| client,
            );
            client.default_database().map_or_else(
                || {
                    tracing::error!("no default database found");
                    std::process::exit(1);
                },
                |db| db,
            )
        });
    }

    pub fn generate_nanoid() -> String {
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        let alphabet = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        nanoid!(20, &alphabet)
    }

    #[derive(Serialize, Default)]
    pub struct ListQueryOptions {
        pub limit: Option<i64>,
        pub skip: Option<u64>,
        pub sort: Option<Document>,
        pub projection: Option<Document>,
    }

    #[derive(Serialize, Default)]
    pub struct FindQueryOptions {
        pub projection: Option<Document>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum Ref<T> {
        Id(String),
        Document(T),
    }

    #[async_trait]
    pub trait Model: Unpin + Serialize + Sized + Send + Sync + DeserializeOwned {
        fn collection_name<'a>() -> &'a str;
        async fn create_indexes() -> Result<Option<CreateIndexesResult>>;

        async fn collection() -> Collection<Self> {
            let name = Self::collection_name();
            DATABASE.get().await.collection::<Self>(name)
        }

        async fn count() -> Result<u64> {
            let count = Self::collection()
                .await
                .estimated_document_count(None)
                .await?;
            Ok(count)
        }

        async fn save(&self) -> Result<&Self> {
            Self::collection().await.insert_one(self, None).await?;
            Ok(self)
        }

        async fn update_one(filter: Document, updates: Document) -> Result<UpdateResult> {
            let updated = Self::collection()
                .await
                .update_one(filter, updates, None)
                .await?;
            Ok(updated)
        }

        async fn update_many(filter: Document, updates: Document) -> Result<UpdateResult> {
            let updated = Self::collection()
                .await
                .update_many(filter, updates, None)
                .await?;
            Ok(updated)
        }

        async fn read(
            filter: Option<Document>,
            options: Option<FindQueryOptions>,
        ) -> Result<Option<Self>, MongoError> {
            let opts = match options {
                Some(opts) => {
                    let options = FindOneOptions::builder()
                        .projection(opts.projection)
                        .build();
                    Some(options)
                }
                None => None,
            };
            Self::collection().await.find_one(filter, opts).await
        }

        async fn list(
            filter: Option<Document>,
            options: Option<ListQueryOptions>,
        ) -> Result<Vec<Self>, MongoError> {
            let opts = match options {
                Some(opts) => {
                    let options = FindOptions::builder()
                        .skip(opts.skip)
                        .limit(opts.limit)
                        .sort(opts.sort)
                        .projection(opts.projection)
                        .build();
                    Some(options)
                }
                None => None,
            };
            let mut result = Self::collection().await.find(filter, opts).await?;
            let mut docs = vec![];
            while let Some(doc) = result.try_next().await? {
                docs.push(doc);
            }
            Ok(docs)
        }

        async fn aggregate(pipeline: &[bson::Document]) -> Result<Vec<Self>> {
            let pipeline = pipeline.to_owned();
            let mut results = Self::collection().await.aggregate(pipeline, None).await?;
            let mut docs = vec![];
            while let Some(doc) = results.try_next().await? {
                let document = bson::from_document(doc)?;
                docs.push(document);
            }
            Ok(docs)
        }

        async fn read_populate<T: DeserializeOwned>(
            query: Document,
            fields: &[&str],
        ) -> Result<Option<T>> {
            let mut pipeline = vec![doc! { "$match": query }];
            for field in fields {
                pipeline.push(doc! {
                    "$lookup": {
                        "from": field,
                        "localField": field,
                        "foreignField": "_id",
                        "as": field
                    }
                });
            }
            pipeline.push(doc! { "$limit": 1 });
            let mut results = Self::collection().await.aggregate(pipeline, None).await?;
            let first = results.try_next().await?;
            if let Some(doc) = first {
                let document = bson::from_document::<T>(doc)?;
                return Ok(Some(document));
            }
            return Ok(None);
        }
    }
}

pub mod prisma;
pub mod prisma_models;
