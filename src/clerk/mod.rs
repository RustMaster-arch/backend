use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, routing::post, extract::Json, Router};
use sqlx::PgPool;

use crate::structs::{User, UserDb};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/add", post(add))
        .route("/delete/:user_id", post(delete))
        .with_state(pool) 
}


pub async fn add(State(pool): State<PgPool>, Json(user_clerk): Json<User>) -> Result<impl IntoResponse, StatusCode> {
    let userdb = UserDb::new(&pool);
    userdb.save_user(user_clerk).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("succesfull");

    Ok(StatusCode::OK)
}

pub async fn delete(State(pool): State<PgPool>, Path(user_id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
     let userdb = UserDb::new(&pool);
    userdb.delete_user(&user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("succesfull");

    Ok(StatusCode::OK)
   
}
