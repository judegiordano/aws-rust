use anyhow::Result;
use argon2::Config;
use async_trait::async_trait;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use slug::slugify;

use super::{PaginationQuery, PrismaHelpers, PRISMA_CLIENT};
use crate::prisma::{address, user, PrismaClient};

#[async_trait]
impl PrismaHelpers<Self> for user::Data {
    async fn paginate(options: PaginationQuery) -> Result<Vec<Self>> {
        let pagination_max = 10;
        let page = options.page.map_or(0, |page| match page {
            1 => 0,
            _ => page,
        });
        let limit = options.limit.map_or(pagination_max, |limit| {
            if limit > pagination_max {
                pagination_max
            } else if limit <= 0 {
                1
            } else {
                limit
            }
        });
        let client = PRISMA_CLIENT.get().await;
        let results = client
            .user()
            .find_many(vec![])
            .with(user::addresses::fetch(vec![]))
            .skip(page * limit)
            .take(limit)
            .exec()
            .await?;
        Ok(results)
    }

    async fn read_by_id(id: &str) -> Result<Option<Self>> {
        let client = PRISMA_CLIENT.get().await;
        let user = client
            .user()
            .find_first(vec![user::id::equals(id.to_string())])
            .with(user::addresses::fetch(vec![]))
            .exec()
            .await?;
        Ok(user)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAddress {
    pub address: i32,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub apt_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUser {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub addresses: Vec<CreateAddress>,
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    let config = Config::default();
    Ok(argon2::hash_encoded(password.as_bytes(), &salt, &config)?)
}

pub fn create_user_slug(first_name: &str, last_name: &str) -> String {
    slugify(format!("{first_name} {last_name}"))
}

pub fn generate_gravatar_hash(email: &str) -> String {
    let email = email.trim().to_lowercase();
    let digest = md5::compute(email.as_bytes());
    format!("{digest:?}")
}

impl user::Data {
    pub async fn create_user(input: CreateUser) -> Result<Self> {
        let client = PRISMA_CLIENT.get().await;
        let avatar_hash = generate_gravatar_hash(&input.email);
        let password = hash_password(&input.password)?;
        let slug = create_user_slug(&input.first_name, &input.last_name);
        let data = client
            .user()
            .create(
                input.email,
                input.first_name,
                input.last_name,
                avatar_hash,
                password,
                slug,
                vec![],
            )
            .exec()
            .await?;
        let addresses = Self::create_address(client, &data.id, input.addresses).await?;
        Ok(Self {
            id: data.id,
            email: data.email,
            first_name: data.first_name,
            last_name: data.last_name,
            avatar_hash: data.avatar_hash,
            password: data.password,
            slug: data.slug,
            created_at: data.created_at,
            updated_at: data.updated_at,
            addresses: Some(addresses),
        })
    }

    pub async fn create_address(
        client: &PrismaClient,
        user_id: &str,
        addresses: Vec<CreateAddress>,
    ) -> Result<Vec<address::Data>> {
        let mut queries = vec![];
        for create_address in addresses {
            let apt_number = if create_address.apt_number.is_some() {
                create_address.apt_number
            } else {
                None
            };
            tracing::warn!(apt_number);
            queries.push(client.address().create(
                create_address.address,
                create_address.street,
                create_address.city,
                create_address.state,
                create_address.zip,
                create_address.country,
                user::id::equals(user_id.to_string()),
                vec![address::apt_number::set(apt_number)],
            ));
        }
        let operations = client._batch(queries).await?;
        Ok(operations)
    }
}
