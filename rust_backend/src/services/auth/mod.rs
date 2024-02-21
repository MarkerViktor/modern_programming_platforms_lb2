use std::ops::DerefMut;
use derive_more::From;
use serde::Serialize;
use sqlx::{query, query_as, Error, Pool, Postgres, Transaction};
use uuid::Uuid;

use crate::services::auth::password_hasher::PasswordHasher;
use crate::services::users::entities::Role;

pub mod entities;
pub mod password_hasher;
pub mod utils;

pub struct AuthService {
    pub db: Pool<Postgres>,
    pub hasher: PasswordHasher,
}

impl AuthService {
    /// Аутентифицировать пользователя по логину и паролю.
    pub async fn login(&self, login: &str, password: &str) -> Result<LoginSuccess, LoginError> {
        let password_hash = self.hasher.get_hash(password);
        let auth = query!(
            r#"
            select uc.user_id, u.role as "role: Role"
            from user_credentials as uc
            join users as u
                on u.id = uc.user_id
            where (uc.login, uc.password_hash) = ($1::text, $2::bytea)"#,
            login,
            &password_hash,
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(LoginError::BadCredentialsError)?;

        let token = Uuid::new_v4().to_string();

        query!(
            r#"
            insert into user_auth (user_id, token)
            values ($1::integer, $2::text)
            on conflict (user_id) do update
            set token = excluded.token"#,
            auth.user_id,
            token,
        )
        .execute(&self.db)
        .await?;

        let login_result = LoginSuccess {
            token,
            role: auth.role,
            user_id: auth.user_id,
        };
        Ok(login_result)
    }

    /// Авторизировать пользователя по токену.
    pub async fn authorize(&self, token: &str) -> Result<UserAuth, AuthError> {
        let auth = query_as!(
            UserAuth,
            r#"
            select
                u.id as user_id,
                u.role as "role: Role"
            from user_auth as ua
            join users as u
                on ua.user_id = u.id
            where ua.token = $1::text"#,
            token,
        )
        .fetch_optional(&self.db)
        .await?;

        auth.ok_or(AuthError::UnknownToken)
    }

    /// Инвалидировать токен пользователя.
    pub async fn logout(&self, user_id: i32) -> Result<(), LogoutError> {
        query!(
            r#"
            delete from user_auth as ua
            where ua.user_id = $1::integer"#,
            user_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Создать данные для аутентификации пользователя.
    pub async fn create_credentials(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        user_id: i32,
        login: &str,
        password: &str,
    ) -> Result<(), CreateCredentialsError> {
        let password_hash = self.hasher.get_hash(password);
        query!(
            r#"
            insert into user_credentials (user_id, login, password_hash)
            values ($1::integer, $2::text, $3::bytea)"#,
            user_id,
            &login,
            &password_hash,
        )
        .execute(transaction.deref_mut())
        .await
        .map_err(|err| match err {
            Error::Database(err) => match err.code().unwrap().as_ref() {
                // Unique violation
                "23505" => CreateCredentialsError::KnownLoginError,
                // Foreign key violation
                "23503" => CreateCredentialsError::UnknownUserIdError,
                _ => Error::Database(err).into(),
            },
            _ => err.into(),
        })?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct LoginSuccess {
    pub token: String,
    pub role: Role,
    pub user_id: i32,
}

#[derive(From, Debug)]
pub enum LoginError {
    BadCredentialsError,
    DbError(Error),
}

#[derive(Debug)]
pub struct UserAuth {
    pub user_id: i32,
    pub role: Role,
}

#[derive(From, Debug)]
pub enum AuthError {
    UnknownToken,
    DbError(Error),
}

#[derive(From, Debug)]
pub enum LogoutError {
    DbError(Error),
}

#[derive(From, Debug)]
pub enum CreateCredentialsError {
    KnownLoginError,
    UnknownUserIdError,
    DbError(Error),
}
