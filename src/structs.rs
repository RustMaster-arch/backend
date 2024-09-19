use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Executor, PgPool};

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct UIQuestion {
    pub question: String,
    pub answers: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Question {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_index: usize,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct ClientRequest {
    pub question_index: usize,
    pub difficulty: String,
    pub answer_index: usize,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub points: i32,
}

pub struct UserDb<'a> {
    pub pool: &'a PgPool
}

impl Question {
    pub fn new(question: &str, answers: Vec<&str>, correct: usize) -> Self {
        let new = answers.iter().map(|answer| answer.to_string()).collect();
        Self { question: question.to_string(), answers: new, correct_index: correct }
    }
}

impl From<Question> for UIQuestion {
    fn from(value: Question) -> Self {
        UIQuestion{question: value.question, answers: value.answers}
    }
}

impl<'a> UserDb<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_user(&self, user: User) -> Result<(), Box<dyn Error>> {
        sqlx::query!("INSERT INTO users (user_id, user_name, points) VALUES ($1, $2, $3)",
            user.user_id,
            user.user_name,
            user.points,
        )
            .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id.to_string()).execute(self.pool).await?;

        Ok(())
    }
}

#[sqlx::test]
async fn test(pool: PgPool) -> sqlx::Result<()> {
    let userdb = UserDb::new(&pool);

    userdb.save_user(User { user_id: "348".to_string(), user_name: "rust".to_string(), points: 500}).await;

    let users = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&pool).await?;

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].user_name, "rust");
    assert_eq!(users[0].user_id, "348");
    assert_eq!(users[0].points, 500);
    
    Ok(())
}
