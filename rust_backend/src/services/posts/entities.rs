use serde::{self, Deserialize, Serialize};
use sqlx::{self, types::chrono::NaiveDateTime};

/// Способы сортировки постов
#[derive(Serialize, Deserialize, Debug)]
pub enum PostsListOrder {
    #[serde(rename = "more_likes_first")]
    MoreLikesFirst,
    #[serde(rename = "more_dislikes_first")]
    ModeDislikesFirst,
    #[serde(rename = "new_first")]
    NewFirst,
}

/// Конфигурация списка постов
#[derive(Serialize, Deserialize, Debug)]
pub struct PostsListConfig {
    #[serde(default = "default_list_conf_page")]
    pub page: u32,
    #[serde(default = "default_list_conf_per_page")]
    pub per_page: u32,
    #[serde(default = "default_list_conf_order")]
    pub order: PostsListOrder,
}

fn default_list_conf_page() -> u32 {
    1
}

fn default_list_conf_per_page() -> u32 {
    5
}

fn default_list_conf_order() -> PostsListOrder {
    PostsListOrder::NewFirst
}

/// Пост ленты
#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Post {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub text: String,
    pub image_url: String,
    #[sqlx(try_from = "i32")]
    pub likes_quantity: u32,
    #[sqlx(try_from = "i32")]
    pub dislikes_quantity: u32,
}

/// Пользовательская оценка поста
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "ratekind")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum RateKind {
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "dislike")]
    Dislike,
}

/// Оценённый пост ленты
#[derive(Serialize)]
pub struct RatedPost {
    #[serde(flatten)]
    pub post: Post,
    pub rate: Option<RateKind>,
}
