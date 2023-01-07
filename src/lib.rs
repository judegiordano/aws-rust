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
    use anyhow::Result;
    use async_once::AsyncOnce;
    use async_trait::async_trait;
    use bson::{doc, Document};
    use futures::stream::TryStreamExt;
    use lazy_static::lazy_static;
    use mongodb::{
        options::ClientOptions,
        results::{CreateIndexesResult, UpdateResult},
        Client, Collection, Database,
    };
    use nanoid::nanoid;
    use serde::{de::DeserializeOwned, Serialize};

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

    #[async_trait]
    pub trait Model: Unpin + Serialize + Sized + Send + Sync + DeserializeOwned {
        fn collection_name<'a>() -> &'a str;
        async fn create_indexes() -> Result<Option<CreateIndexesResult>>;

        async fn collection() -> Collection<Self> {
            let name = Self::collection_name();
            DATABASE.get().await.collection::<Self>(name)
        }

        async fn read_by_id(_id: &str) -> Result<Self> {
            let filter = doc! { "_id": _id };
            let result = Self::collection().await.find_one(filter, None).await?;
            match result {
                Some(doc) => Ok(doc),
                None => anyhow::bail!(format!("{} document not found", Self::collection_name())),
            }
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

        async fn update(filter: Document, doc: Document) -> Result<UpdateResult> {
            let now = chrono::Utc::now();
            let updates = vec![doc! { "$set": doc }, doc! { "$set": { "updated_at": now } }];
            let updated = Self::collection()
                .await
                .update_one(filter, updates, None)
                .await?;
            Ok(updated)
        }

        async fn list() -> Result<Vec<Self>> {
            let mut result = Self::collection().await.find(None, None).await?;
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
                docs.push(bson::from_document(doc)?);
            }
            Ok(docs)
        }
    }
}
