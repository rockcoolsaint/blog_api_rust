use actix_web::{dev::AppConfig, post, web, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::utils::{api_response::{self, ApiResponse}, app_state, jwt::encode_jwt};

#[derive(Serialize, Deserialize)]
struct RegisterModel {
  name: String,
  email: String,
  password: String
}

#[derive(Serialize, Deserialize)]
struct LoginModel {
  email: String,
  password: String
}

#[post("/register")]
pub async fn register(
  app_state: web::Data<app_state::AppState>,
  register_json: web::Json<RegisterModel>
) -> Result<ApiResponse, ApiResponse> {
  let user_model = entity::user::ActiveModel {
    name: Set(register_json.name.clone()),
    email: Set(register_json.email.clone()),
    password: Set(digest(&register_json.password)),
    ..Default::default()
  }.insert(&app_state.db).await
  .map_err(|err| ApiResponse::new(500, err.to_string()))?;

  Ok(api_response::ApiResponse::new(200, format!("{}", user_model.id)))
}

#[post("/login")]
pub async fn login(
  app_state: web::Data<app_state::AppState>,
  login_json: web::Json<LoginModel>
) -> Result<ApiResponse, ApiResponse> {
  let user_data = entity::user::Entity::find()
    .filter(
      Condition::all()
      .add(entity::user::Column::Email.eq(&login_json.email))
      .add(entity::user::Column::Password.eq(digest(&login_json.password)))
    ).one(&app_state.db).await
    .map_err(|err| ApiResponse::new(500, err.to_string()))?
    .ok_or(ApiResponse::new(404, "User Not Found".to_owned()))?;

  // if user.is_none() {
  //   return api_response::ApiResponse::new(401, "User Not Found".to_string())
  // }

  // let user_data = user.unwrap();

  let token = encode_jwt(user_data.email, user_data.id)
  .map_err(|err| ApiResponse::new(500, err.to_string()))?;

  Ok(api_response::ApiResponse::new(200, format!("{{ 'token':'{}' }}", token)))
}