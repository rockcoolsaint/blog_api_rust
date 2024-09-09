use actix_web::web;
use actix_web_lab::middleware::from_fn;

use super::{handlers, middlewares};
// use crate::routes::handlers; this can be used in place of the use super::handlers line above


pub fn config(config: &mut web::ServiceConfig) {
  config
  .service(
    web::scope("/user")
    .wrap(from_fn(middlewares::auth_middleware::check_auth_middleware))
    .service(handlers::user_handlers::user)
  );
}