use actix_web::web::{self, service};
use actix_web_lab::middleware::from_fn;

use super::{handlers::{self, post_handlers}, middlewares};
// use crate::routes::handlers; this can be used in place of the use super::handlers line above


pub fn config(config: &mut web::ServiceConfig) {
  config
  .service(
    web::scope("secure/post")
    .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
    .service(post_handlers::create_post)
    .service(post_handlers::all_posts)
  )
  .service(
    web::scope("/post")
    .service(post_handlers::one_posts)
    .service(post_handlers::all_posts)
  );
}