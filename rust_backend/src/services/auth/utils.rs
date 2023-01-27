use std::fmt::Debug;

use actix_session::{Session, SessionExt};
use actix_web::{web, HttpRequest};
use derive_more::From;

use crate::services::auth::{AuthService, UserAuth};
use crate::services::users::entities::Role;

pub trait GetSetToken {
    fn get_auth_token(&self) -> Option<String>;
    fn set_auth_token(&self, value: &String);
}

impl GetSetToken for HttpRequest {
    fn get_auth_token(&self) -> Option<String> {
        self.get_session().get("auth_token").unwrap()
    }

    fn set_auth_token(&self, value: &String) {
        self.get_session()
            .insert("auth_token", value.clone())
            .unwrap()
    }
}

pub async fn get_auth(request: &HttpRequest) -> Option<UserAuth> {
    let token = request.get_auth_token()?;
    let auth_service = request.app_data::<web::Data<AuthService>>().unwrap();
    match auth_service.authorize(&token).await {
        Ok(user_auth) => Some(user_auth),
        Err(_) => None,
    }
}

pub async fn get_auth_or_guest(request: &HttpRequest) -> UserAuth {
    match get_auth(request).await {
        Some(auth) => auth,
        None => UserAuth {
            user_id: 0,
            role: Role::Guest,
        },
    }
}

pub async fn check_auth_role(roles: &[Role], request: &HttpRequest) -> Result<UserAuth, AuthError> {
    let auth = get_auth_or_guest(request).await;
    if roles.iter().any(|r| *r == auth.role) {
        Ok(auth)
    } else {
        Err(AuthError::BadRoleError)
    }
}

#[derive(Debug, From)]
pub enum AuthError {
    BadRoleError,
}
