use std::path::PathBuf;

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{get, post, web};
use chrono::{NaiveDateTime, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{api_response, app_state, constants, jwt::Claims};

#[derive(MultipartForm)]
struct CreatePostModel {
    title: Text<String>,
    text: Text<String>,
    file: TempFile
}

#[derive(Deserialize, Serialize)]
struct PostModel {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub uuid: Uuid,
    pub image: Option<String>,
    pub userid: i32,
    pub createdat: NaiveDateTime,
    pub user: Option<UserModel>
}

#[derive(Deserialize, Serialize)]
struct UserModel {
    name: String,
    email: String
}

#[post("create")]
pub async fn create_post(
  app_state: web::Data<app_state::AppState>,
  claim: Claims,
  post_model: MultipartForm<CreatePostModel>
) -> Result<api_response::ApiResponse, api_response::ApiResponse> {

    let check_name = post_model.file.file_name.clone().unwrap_or("null".to_owned());
    let max_file_size = (*constants::MAX_FILE_SIZE).clone();

    match &check_name[check_name.len() - 4 ..] {
        ".png" | ".jpg" => {},
        _ => {
            return Err(api_response::ApiResponse::new(400, " Invald File type".to_owned()))
        }
    }

    match post_model.file.size {
        0 => {
            return Err(api_response::ApiResponse::new(400, " Invald File type".to_owned()))
        },
        length if length > max_file_size as usize => {
            return Err(api_response::ApiResponse::new(401, " File Too Big".to_owned()))

        },
        _ => {

        }
    }

    let txn = app_state.db.begin().await
    .map_err(|err| api_response::ApiResponse::new(500,err.to_string()))?;

    let post_entity = entity::post::ActiveModel
    {
        title: Set(post_model.title.clone()),
        text: Set(post_model.text.clone()),
        uuid: Set(Uuid::new_v4()),
        userid: Set(claim.id),
        createdat: Set(Utc::now().naive_local()),
        ..Default::default()
    };

    let mut created_entity = post_entity.save(&txn).await
    .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?;

    let temp_file_path = post_model.file.file.path();
    let file_name = post_model.file.file_name.as_ref()
    .map(|m| m.as_ref())
    .unwrap_or("null");

    let time_stamp = Utc::now().timestamp();

    let mut file_path = PathBuf::from("./public");
    let new_file_name = format!("{}-{}", time_stamp, file_name);
    file_path.push(&new_file_name);

    match std::fs::copy(temp_file_path, file_path) {
        Ok(_) => {

            created_entity.image = Set(Some(new_file_name));
            created_entity.update(&txn).await
            .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?;

            txn.commit().await
            .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?;

            Ok(api_response::ApiResponse::new(200, "Post Created".to_owned()))
        }
        Err(_) => {
        txn.rollback().await
        .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?;

        Err(api_response::ApiResponse::new(500, "Internal Server Error".to_owned()))
        }
    }

}

#[get("my-posts")]
pub async fn my_posts(
  app_state: web::Data<app_state::AppState>,
  claim: Claims
) -> Result<api_response::ApiResponse, api_response::ApiResponse> {

  let posts: Vec<PostModel> = entity::post::Entity::find()
  .filter(entity::post::Column::Userid.eq(claim.id)).all(&app_state.db).await
  .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?
  .into_iter()
  .map(|post| 
    PostModel {
      id: post.id,
      title: post.title,
      text: post.text,
      uuid: post.uuid,
    //   image: post.image.expect("Post should have an image"),
      image: post.image,
      userid: post.userid,
      createdat: post.createdat,
      user: None
    }
  ).collect();
  let res_str = serde_json::to_string(&posts)
  .map_err(|err| api_response::ApiResponse::new(500,err.to_string()))?;

  Ok(api_response::ApiResponse::new(200, res_str.to_owned()))
}

#[get("all-posts")]
pub async fn all_posts(
  app_state: web::Data<app_state::AppState>
) -> Result<api_response::ApiResponse, api_response::ApiResponse> {

  let posts: Vec<PostModel> = entity::post::Entity::find()
  .all(&app_state.db).await
  .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?
  .into_iter()
  .map(|post| 
    PostModel {
      id: post.id,
      title: post.title,
      text: post.text,
      uuid: post.uuid,
    //   image: post.image.expect("Post should have an image"),
      image: post.image,
      userid: post.userid,
      createdat: post.createdat,
      user: None
    }
  ).collect();
  let res_str = serde_json::to_string(&posts)
  .map_err(|err| api_response::ApiResponse::new(500,err.to_string()))?;

  Ok(api_response::ApiResponse::new(200, res_str.to_owned()))
}

#[get("post/{post_uuid}")]
pub async fn one_post(
  app_state: web::Data<app_state::AppState>,
  post_uuid: web::Path<Uuid>
) -> Result<api_response::ApiResponse, api_response::ApiResponse> {

    let posts: PostModel = entity::post::Entity::find()
    .filter(entity::post::Column::Uuid.eq(post_uuid.clone()))
    .find_also_related(entity::user::Entity)
    .one(&app_state.db).await
    .map_err(|err| api_response::ApiResponse::new(500, err.to_string()))?
    .map(|post| 
        PostModel {
            id: post.0.id,
            title: post.0.title,
            text: post.0.text,
            uuid: post.0.uuid,
            //   image: post.image.expect("Post should have an image"),
            image: post.0.image,
            userid: post.0.userid,
            createdat: post.0.createdat,
            user: post.1.map(|item| UserModel { name: item.name, email: item.email })
        }
    )
    .ok_or(api_response::ApiResponse::new(400, "No Post Found".to_string()))?;
    let res_str = serde_json::to_string(&posts)
    .map_err(|err| api_response::ApiResponse::new(500,err.to_string()))?;

  Ok(api_response::ApiResponse::new(200, res_str.to_owned()))
}