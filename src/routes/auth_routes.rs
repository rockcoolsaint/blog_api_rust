use actix_web::web;

use super::handlers;
// use crate::routes::handlers; this can be used in place of the use super::handlers line above


pub fn config(config: &mut web::ServiceConfig) {
  config
  .service(
    web::scope("/auth")
    .service(handlers::auth_handlers::register)
    .service(handlers::auth_handlers::login)
  );
}