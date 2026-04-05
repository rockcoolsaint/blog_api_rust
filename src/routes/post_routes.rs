use actix_web::{middleware::{self, from_fn}, web};

use crate::routes::middlewares;

use super::handlers;

pub fn config(config: &mut web::ServiceConfig) {
    config
    .service(
        web::scope("secure/post")
        .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
        .service(handlers::post_handler::create_post)
        .service(handlers::post_handler::my_posts)
    )
    .service(
        web::scope("/post")
        .service(handlers::post_handler::one_post)
        .service(handlers::post_handler::all_posts)
    );
}