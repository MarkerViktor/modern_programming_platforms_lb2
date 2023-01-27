use crate::services::users::entities::Role;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LoginSuccess {
    pub user_id: i32,
    pub token: String,
    pub role: Role,
}

#[derive(Debug)]
pub struct AuthSuccess {
    pub is_authorized: bool,
    pub role: Role,
    pub user_id: i32,
}
