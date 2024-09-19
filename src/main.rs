use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clerk::router;
use dotenvy::dotenv;
use questions::{easy, hard, medium, very_hard};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use std::{env, sync::Arc, time::Duration};
mod structs;
mod questions;
mod clerk;
use structs::{ClientRequest, Question, UIQuestion};

struct AppState {
    easy: Vec<Question>,
    medium: Vec<Question>,
    hard: Vec<Question>,
    very_hard: Vec<Question>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let easy: Vec<Question> = easy();
    let medium: Vec<Question> = medium();
    let hard: Vec<Question> = hard();
    let very_hard: Vec<Question> = very_hard(); 

    let app_state = Arc::new(AppState {
        easy,
        medium,
        hard,
        very_hard,
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("can't connect to database");


    let cors = CorsLayer::permissive();

    let user_creation_router = router(pool);

    let app = Router::new()
        .route("/:difficulty", get(questions))
        .route("/correct", get(correct))
        .with_state(app_state)
        .nest("/user", user_creation_router)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn questions(Path(difficulty): Path<String>, State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    let questions = match difficulty.as_str() {
        "easy" => &state.easy,
        "medium" => &state.medium,
        "hard" => &state.hard,
        "very_hard" => &state.very_hard,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };

    let ui_questions: Vec<UIQuestion> = questions.iter().map(|q| UIQuestion::from(q.clone())).collect();
    
    Ok(Json(ui_questions))
}

async fn correct(State(state): State<Arc<AppState>>, Query(ui_question): Query<ClientRequest>) -> Result<impl IntoResponse, StatusCode> {
    let questions = match ui_question.difficulty.as_str() {
        "easy" => &state.easy,
        "medium" => &state.medium,
        "hard" => &state.hard,
        "very_hard" => &state.very_hard,
        _ => return Err(StatusCode::NOT_ACCEPTABLE),
    };

    let result = questions.iter().find_map(|question| {
        if question.question == questions[ui_question.question_index].question && 
        question.answers[ui_question.answer_index] == question.answers[question.correct_index] {
            return Some(true)
        } else {
            return None
        }
    }).unwrap_or(false);

    Ok((StatusCode::OK, Json(result)))
}
