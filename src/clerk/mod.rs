use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{extract::State, routing::post, extract::Json, Router};
use serde_json::to_string;
use sqlx::PgPool;

use crate::structs::{StatsUi, User, UserDb};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/add", post(add))
        .route("/delete/:user_id", post(delete))
        .route("points/:user_id", get(get_points_request))
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

pub async fn get_points_request(State(pool): State<PgPool>, Path(user_id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let userdb = UserDb::new(&pool);
    let points = userdb.get_poits(&user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    let points = to_string(&points.unwrap()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    Ok((StatusCode::OK, points))
}
