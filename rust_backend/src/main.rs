use std::path::Path;

use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use actix_web::cookie::Key;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use crate::handlers::auth;
use crate::handlers::posts;
use crate::services::auth::AuthService;
use crate::services::auth::password_hasher::PasswordHasher;
use crate::services::posts::image_storage::ImageStorage;
use crate::services::posts::PostsService;
use crate::services::users::UsersService;

mod handlers;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let address = std::env::var("ADDRESS").expect("ADDRESS must be set.");
    let cookie_key = std::env::var("COOKIE_KEY").expect("Cookie secure key must be set.");
    let cookie_domain = std::env::var("COOKIE_DOMAIN").expect("COOKIE_DOMAIN must be set.");
    let storage_path = std::env::var("STORAGE_PATH").expect("STORAGE_PATH must be set.");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let pool = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Error building a connection pool");

    let auth_service = Data::new(AuthService {
        db: pool.clone(),
        hasher: PasswordHasher::from_env(),
    });
    let posts_service = Data::new(PostsService {
        db: pool.clone(),
        storage: ImageStorage {
            root: Path::new(&storage_path).to_path_buf(),
        },
    });
    let users_service = Data::new(UsersService {
        db: pool.clone(),
        auth_service: auth_service.clone(),
    });

    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(
                CookieSessionStore::default(),
                Key::try_from(cookie_key.as_bytes()).expect("COOKIE_KEY too short (min 64)."),
            )   
                .cookie_domain(Some(cookie_domain.to_string()))
                .cookie_http_only(true)
                .cookie_secure(false)
                .build();

        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .wrap(session_middleware)
            .app_data(auth_service.clone())
            .app_data(posts_service.clone())
            .app_data(users_service.clone())
            .service(auth::sign_in)
            .service(auth::sign_up)
            .service(auth::sign_out)
            .service(posts::get_posts)
            .service(posts::create_post)
            .service(posts::set_post_rate)
    })
        .bind(address)
        .expect("Provided address is occupied for binding.")
        .run()
        .await
}
