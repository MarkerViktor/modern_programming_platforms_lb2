pub mod entities;
pub mod image_storage;

use derive_more::From;
use sqlx::{query, query_as, Error, Pool, Postgres};
use std::collections::HashMap;
use std::ops::DerefMut;

use crate::services::posts::entities::{Post, PostsListConfig, PostsListOrder, RateKind};
use crate::services::posts::image_storage::{ImageStorage, SaveImageError};

pub struct PostsService {
    pub db: Pool<Postgres>,
    pub storage: ImageStorage,
}

impl PostsService {
    /// Получить список постов.
    pub async fn get_posts(&self, list_conf: &PostsListConfig) -> Result<Vec<Post>, GetPostsError> {
        let order_by_column = match list_conf.order {
            PostsListOrder::MoreLikesFirst => "p.likes_quantity desc, p.created_at desc",
            PostsListOrder::ModeDislikesFirst => "p.dislikes_quantity, p.created_at desc",
            PostsListOrder::NewFirst => "p.created_at desc",
        };

        let query_with_order = format!(
            r"
            select
                p.id, p.created_at, p.text, p.image_url,
                p.likes_quantity, p.dislikes_quantity
            from posts as p
            order by {}
            limit $1::integer
            offset $2::integer
            ",
            order_by_column,
        );

        let limit = list_conf.per_page;
        let offset = (list_conf.page - 1) * list_conf.per_page;
        let posts = query_as::<_, Post>(&query_with_order)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&self.db)
            .await?;
        Ok(posts)
    }

    /// Получить оценки пользователя для заданных постов.
    pub async fn get_posts_rates(
        &self,
        posts_ids: &Vec<i32>,
        user_id: i32,
    ) -> Result<HashMap<i32, RateKind>, GetPostsRatesError> {
        let records = query!(
            r#"
            select
                pr.post_id,
                pr.rate as "rate: RateKind"
            from post_rates as pr
            join posts as p
                on pr.post_id = p.id
            join users as u
                on pr.user_id = u.id
            where pr.user_id = $1::integer
                and pr.post_id = any($2::integer[])"#,
            user_id,
            posts_ids,
        )
        .fetch_all(&self.db)
        .await?;
        let rates_map = records
            .iter()
            .map(|r| (r.post_id, r.rate.clone()))
            .collect::<HashMap<_, _>>();
        Ok(rates_map)
    }

    /// Получить количество всех постов.
    pub async fn get_total_posts_quantity(&self) -> Result<i64, GetPostsQuantityError> {
        let count = query!("select count(*) from posts")
            .fetch_one(&self.db)
            .await?
            .count;
        Ok(count.unwrap())
    }

    pub async fn create_new_post(
        &self,
        text: &str,
        image_bytes: &[u8],
    ) -> Result<Post, CreatePostError> {
        let file_name = self.storage.save(image_bytes)?;
        let image_path = format!("/storage/{}", file_name);

        let record = query!(
            r"
            insert into posts (text, image_url)
            values ($1::text, $2::text)
            returning id, created_at",
            text,
            &image_path,
        )
        .fetch_one(&self.db)
        .await?;
        let post = Post {
            id: record.id,
            created_at: record.created_at,
            text: String::from(text),
            image_url: image_path,
            likes_quantity: 0,
            dislikes_quantity: 0,
        };
        Ok(post)
    }

    pub async fn update_user_rate(
        &self,
        post_id: i32,
        user_id: i32,
        new_rate: Option<RateKind>,
    ) -> Result<(), UpdateRateError> {
        // check user_id and post_id exists

        let mut transaction = self.db.begin().await?;

        let check = query!(
            r"
            select exists(select 0 from users as u where u.id = $1::integer)::boolean as user_exist,
                   exists(select 0 from posts as p where p.id = $2::integer)::boolean as post_exist
            ",
            user_id,
            post_id,
        )
        .fetch_one(transaction.deref_mut())
        .await?;

        if !check.user_exist.unwrap() {
            return Err(UpdateRateError::UnknownUserId);
        }
        if !check.post_exist.unwrap() {
            return Err(UpdateRateError::UnknownPostId);
        }

        // Get old rate.
        let record = query!(
            r#"
            select rate as "rate: RateKind"
            from post_rates as pr
            where (pr.post_id, pr.user_id) = ($1::integer, $2::integer)
            "#,
            post_id,
            user_id,
        )
        .fetch_optional(transaction.deref_mut())
        .await?;
        let old_rate: Option<RateKind> = match record {
            Some(record) => Some(record.rate),
            None => None,
        };

        if old_rate == new_rate {
            return Ok(());
        }

        // Calculate likes and dislikes dif.
        let mut likes = 0;
        let mut dislikes = 0;

        if let Some(old_rate) = old_rate {
            match old_rate {
                RateKind::Like => likes -= 1,
                RateKind::Dislike => dislikes -= 1,
            }
        }
        if let Some(new_rate) = new_rate {
            match new_rate {
                RateKind::Like => likes += 1,
                RateKind::Dislike => dislikes += 1,
            }
        }

        // Update post's likes and dislikes quantity.
        query!(
            r"
            update posts
            set likes_quantity = likes_quantity + $2::integer,
                dislikes_quantity = dislikes_quantity + $3::integer
            where id = $1::integer
            ",
            post_id,
            likes,
            dislikes,
        )
        .execute(transaction.deref_mut())
        .await?;

        // Create, update or delete user's post reaction.
        match new_rate {
            Some(new_rate) => {
                query!(
                    r"
                    insert into post_rates (post_id, user_id, rate)
                    values ($1::integer, $2::integer, $3::rate_kind_t)
                    on conflict (post_id, user_id) do update
                    set rate = excluded.rate
                    ",
                    post_id,
                    user_id,
                    new_rate as RateKind,
                )
                .execute(transaction.deref_mut())
                .await?
            }
            None => {
                query!(
                    r"
                    delete from post_rates as pr
                    where pr.user_id = $1",
                    user_id,
                )
                .execute(transaction.deref_mut())
                .await?
            }
        };

        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, From)]
pub enum GetPostsError {
    DbError(Error),
}

#[derive(Debug, From)]
pub enum GetPostsRatesError {
    DbError(Error),
}

#[derive(Debug, From)]
pub enum GetPostsQuantityError {
    DbError(Error),
}

#[derive(Debug, From)]
pub enum CreatePostError {
    SaveImageError(SaveImageError),
    DbError(Error),
}

#[derive(Debug, From)]
pub enum UpdateRateError {
    UnknownUserId,
    UnknownPostId,
    Db(Error),
}

