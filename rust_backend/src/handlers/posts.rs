use actix_multipart_extract::{File, Multipart, MultipartForm};
use actix_web::{route, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;
use std::str;

use crate::services::auth::utils::check_auth_role;
use crate::services::posts::entities::{PostsListConfig, RateKind, RatedPost};
use crate::services::posts::PostsService;
use crate::services::users::entities::Role;

#[derive(Serialize)]
pub struct PostsList {
    items: Vec<RatedPost>,
    total_quantity: i32,
}

/// Получить список постов с оценками пользователя, выполняющего запрос.
#[route("/posts", method = "GET")]
pub async fn get_posts(
    request: HttpRequest,
    posts_list_config: web::Query<PostsListConfig>,
    posts_service: web::Data<PostsService>,
) -> impl Responder {
    let user_id = match check_auth_role(&[Role::User, Role::Admin], &request).await {
        Ok(auth) => auth.user_id,
        Err(_) => {
            return HttpResponse::Forbidden()
                .body("Accessible only for users with roles User or Admin.")
        }
    };

    let posts = match posts_service.get_posts(&posts_list_config).await {
        Ok(posts) => posts,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let posts_ids: Vec<i32> = posts.iter().map(|p| p.id).collect();

    let mut rates_map = match posts_service.get_posts_rates(&posts_ids, user_id).await {
        Ok(rates_map) => rates_map,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let rated_posts: Vec<RatedPost> = posts
        .iter()
        .map(|post| RatedPost {
            post: post.clone(),
            rate: rates_map.remove(&post.id),
        })
        .collect();

    let total_posts_quantity = match posts_service.get_total_posts_quantity().await {
        Ok(quantity) => quantity as i32,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(PostsList {
        items: rated_posts,
        total_quantity: total_posts_quantity,
    })
}

#[derive(Debug, Deserialize, MultipartForm)]
pub struct CreatePostPayload {
    #[multipart(max_size = 10 MB)]
    image: File,
    text: String,
}

/// Создать новый пост.
#[route("/posts", method = "POST")]
pub async fn create_post(
    payload: Multipart<CreatePostPayload>,
    request: HttpRequest,
    posts_service: web::Data<PostsService>,
) -> impl Responder {
    if let Err(_) = check_auth_role(&[Role::Admin], &request).await {
        return HttpResponse::Forbidden().body("Accessible only for users with role Admin.");
    };

    match payload.image.content_type.as_str() {
        "image/jpeg" | "image/png" => {}
        _ => {
            return HttpResponse::BadRequest()
                .body("Bad image Content-Type. Only image/jpeg image/png allowed.")
        }
    }

    match posts_service
        .create_new_post(&payload.text, payload.image.bytes.as_ref())
        .await
    {
        Ok(post) => HttpResponse::Ok().json(&post),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetPostRatePayload {
    pub rate: Option<RateKind>,
}

#[route("/posts/{post_id}/rate", method = "PUT")]
pub async fn set_post_rate(
    path: web::Path<i32>,
    payload: web::Json<SetPostRatePayload>,
    request: HttpRequest,
    posts_service: web::Data<PostsService>,
) -> impl Responder {
    let user_id = match check_auth_role(&[Role::User], &request).await {
        Ok(auth) => auth.user_id,
        _ => return HttpResponse::Forbidden().body("Accessible only for authorized users."),
    };
    let post_id = path.into_inner();

    match posts_service
        .update_user_rate(post_id, user_id, payload.rate)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
