use actix_web::{route, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use std::str;

use crate::services::auth::utils::{check_auth_role, GetSetToken};
use crate::services::auth::{AuthService, LoginError};
use crate::services::users::entities::Role;
use crate::services::users::{RegistrationError, UsersService};

#[derive(Deserialize)]
pub struct SignInPayload {
    pub login: String,
    pub password: String,
}

/// Аутентификация по логину и паролю с установкой токена в Cookie.
#[route("/sign_in", method = "POST")]
pub async fn sign_in(
    request: HttpRequest,
    payload: web::Form<SignInPayload>,
    auth_service: web::Data<AuthService>,
) -> impl Responder {
    if let Err(_) = check_auth_role(&[Role::Guest], &request).await {
        return HttpResponse::Forbidden().body("Accessible only for users with role Guest.");
    }
    match auth_service.login(&payload.login, &payload.password).await {
        Ok(result) => {
            request.set_auth_token(&result.token);
            HttpResponse::Ok().json(result)
        }
        Err(err) => match err {
            LoginError::BadCredentialsError => {
                HttpResponse::BadRequest().json("Неверный логин или пароль.")
            }
            LoginError::DbError(_) => HttpResponse::InternalServerError().finish(),
        },
    }
}

#[derive(Deserialize)]
pub struct SignUpPayload {
    pub first_name: String,
    pub last_name: String,
    pub login: String,
    pub password: String,
}

/// Зарегистрировать нового пользователя.
#[route("/sign_up", method = "POST")]
pub async fn sign_up(
    request: HttpRequest,
    payload: web::Form<SignUpPayload>,
    users_service: web::Data<UsersService>,
) -> impl Responder {
    if let Err(_) = check_auth_role(&[Role::Guest], &request).await {
        return HttpResponse::Forbidden().body("Accessible only for users with role Guest.");
    }
    match users_service
        .registration(
            &payload.first_name,
            &payload.last_name,
            &payload.login,
            &payload.password,
        )
        .await
    {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(RegistrationError::LoginAlreadyOccupied) => {
            HttpResponse::BadRequest().json("Введённый логин уже занят.")
        }
        _ => HttpResponse::InternalServerError().finish(),
    }
}

/// Выйти из аккаунта пользователя (удалить токен из Cookie).
#[route("/sign_out", method = "POST")]
pub async fn sign_out(
    request: HttpRequest,
    auth_service: web::Data<AuthService>,
) -> impl Responder {
    let user_id = match check_auth_role(&[Role::User, Role::Admin], &request).await {
        Ok(auth) => auth.user_id,
        _ => return HttpResponse::Forbidden().body("Accessible only for authorized users."),
    };
    match auth_service.logout(user_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}
