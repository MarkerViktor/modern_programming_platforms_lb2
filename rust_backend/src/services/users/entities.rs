use serde::{self, Serialize};
use sqlx;

#[derive(Debug, sqlx::Type, Serialize, Eq, PartialEq, Clone, Copy)]
#[sqlx(type_name = "user_role_t")]
pub enum Role {
    #[sqlx(rename = "USER")]
    #[serde(rename = "user")]
    User,
    #[sqlx(rename = "ADMIN")]
    #[serde(rename = "admin")]
    Admin,
    #[sqlx(rename = "ADMIN")]
    #[serde(rename = "admin")]
    Guest,
}
