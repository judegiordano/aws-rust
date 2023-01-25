use anyhow::Result;
use async_trait::async_trait;

use super::{PaginationQuery, PrismaHelpers, PRISMA_CLIENT};
use crate::prisma::user;

#[async_trait]
impl PrismaHelpers<user::Data> for user::Data {
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
