use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Executor, PgPool};

const EASY_MULTIPLIER: f32 = 1.2;
const MEDIUM_MULTIPLIER: f32 = 1.4;
const HARD_MULTIPLIER: f32 = 1.75;
const VERY_HARD_MULTIPLIER: f32 = 2.75;

const CORRECT_POINTS: u32 = 10;

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
    pub user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct User {
    pub user_id: String,
    pub user_name: String,
}

pub struct StatsUi {
    pub user_id: String,
    pub correct_answers: u32,
    pub difficulty: String,
}

pub struct UserU {
    pub user_id: String,
    pub user_name: String,
    pub points: i32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct UserLeaderBoard {
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
        sqlx::query!("INSERT INTO users (user_id, user_name) VALUES ($1, $2)",
            user.user_id,
            user.user_name,
        )
            .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<(), Box<dyn Error>> {
        sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id.to_string()).execute(self.pool).await?;

        Ok(())
    }

    pub async fn add_points_to_user(&self, user_id: &str, points: i32) -> Result<(), Box<dyn Error>> {
        println!("updating points 3");

        let prev_points = sqlx::query_scalar!("SELECT points FROM users WHERE user_id = $1", user_id).fetch_one(self.pool).await?;

        sqlx::query!("UPDATE users SET points = $1 WHERE user_id = $2", points + prev_points, user_id).execute(self.pool).await?;

        Ok(())
    }

    pub async fn get_poits(&self, user_id: &str) -> Result<i32, Box<dyn Error>> {
        let points = sqlx::query_scalar!("SELECT points FROM users WHERE user_id = $1", user_id).fetch_one(self.pool).await?;

        Ok(points)
    }

    pub async fn leader_board(&self) -> Result<Vec<UserLeaderBoard>, Box<dyn Error>> {
        let users = sqlx::query_as!(UserLeaderBoard, "SELECT user_name, points FROM users ORDER BY points DESC").fetch_all(self.pool).await?;

        Ok(users)
    }
}

impl StatsUi {
    pub fn new(correct_answers: u32, difficulty: &str, user_id: &str) -> Self {
        Self { user_id: user_id.to_string(), correct_answers, difficulty: difficulty.to_string() }
    }

    pub fn get_points(&self) -> u32 {
        let current_correct_points = CORRECT_POINTS * self.correct_answers;

        match self.difficulty.as_str() {
            "easy" => (current_correct_points as f32 * EASY_MULTIPLIER) as u32,
            "medium" => (current_correct_points as f32 * MEDIUM_MULTIPLIER) as u32,
            "hard" => (current_correct_points as f32 * HARD_MULTIPLIER) as u32,
            "very_hard" => (current_correct_points as f32 * VERY_HARD_MULTIPLIER) as u32,
            _ => 0,
        }
    }

    pub async fn update_points(&self, pool: PgPool) -> Result<(), Box<dyn Error>> {
        let userdb = UserDb::new(&pool);
        let points = self.get_points() as i32;

        println!("updating points 2");
        
        userdb.add_points_to_user(&self.user_id, points).await;
        
        Ok(())
    }
}

#[sqlx::test]
async fn test(pool: PgPool) -> sqlx::Result<()> {
    let userdb = UserDb::new(&pool);

    userdb.save_user(User { user_id: "348".to_string(), user_name: "rust".to_string()}).await;

    let users = sqlx::query_as!(User, "SELECT user_name, user_id FROM users")
        .fetch_all(&pool).await?;

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].user_name, "rust");
    assert_eq!(users[0].user_id, "348");
    
    Ok(())
}

#[sqlx::test]
async fn update_test(pool: PgPool) -> sqlx::Result<()> {
    let userdb = UserDb::new(&pool);

    userdb.save_user(User { user_id: "38".to_string(), user_name: "rust".to_string()}).await;

    let stats = StatsUi::new(3, "easy", "38");
    
    stats.update_points(pool.clone()).await;

    let users = sqlx::query_as!(UserU, "SELECT * FROM users")
        .fetch_all(&pool).await?;

    assert_eq!(users[0].points, 30);

    Ok(())
}
