pub mod entities;

use std::ops::DerefMut;
use crate::services::auth::{AuthService, CreateCredentialsError};
use crate::services::users::entities::Role;
use actix_web::web::Data;
use derive_more::From;
use sqlx::{self, query, Pool, Postgres};

pub struct UsersService {
    pub db: Pool<Postgres>,
    pub auth_service: Data<AuthService>,
}

impl UsersService {
    /// Зарегистрировать пользователя.
    pub async fn registration(
        &self,
        first_name: &String,
        last_name: &String,
        login: &String,
        password: &String,
    ) -> Result<(), RegistrationError> {
        let mut transaction = self.db.begin().await?;

        let user = query!(
            r#"
            insert into users (first_name, last_name, role)
            values ($1::text, $2::text, $3::user_role_t) returning id
            "#,
            &first_name,
            &last_name,
            Role::User as Role,
        )
        .fetch_one(transaction.deref_mut())
        .await?;

        self.auth_service
            .create_credentials(&mut transaction, user.id, login, password)
            .await
            .map_err(|err| match err {
                CreateCredentialsError::KnownLoginError => RegistrationError::LoginAlreadyOccupied,
                _ => err.into(),
            })?;

        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, From)]
pub enum RegistrationError {
    LoginAlreadyOccupied,
    CreateCredentialsError(CreateCredentialsError),
    DbError(sqlx::Error),
}
