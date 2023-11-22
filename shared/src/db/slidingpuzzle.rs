// CREATE TABLE IF NOT EXISTS slidingpuzzle (
//     -- singleplayer game, so we will be keeping track of the users score and time
//     id SERIAL PRIMARY KEY,
//     user_id integer NOT NULL REFERENCES users(id),
//     difficulty integer NOT NULL, -- 1 = easy, 2 = medium, 3 = hard
//     size integer NOT NULL, -- 3 = 3x3, 4 = 4x4, 5 = 5x5
//     score integer[] NOT NULL, -- The users list of scores for this size
//     time integer[] NOT NULL, -- The users list of times for this size
//     created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//     updated_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
// );

use sqlx::{types::chrono, Acquire};

use crate::anyhow::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct SlidingPuzzle {
    id: i64,
    pub user_id: i64,
    pub difficulty: i32,
    pub size: i32,
    pub score: i32,
    pub time: i32,
    created_at: chrono::NaiveDateTime,
}

impl SlidingPuzzle {
    pub async fn create(user_id: i32, difficulty: i32, size: i32, score: i32, time: i32, db: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
        let mut tx = db.begin().await?;

        let puzzle = sqlx::query_as!(
            SlidingPuzzle,
            r#"
            INSERT INTO slidingpuzzle (user_id, difficulty, size, score, time)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            user_id,
            difficulty,
            size,
            score,
            time
        )
        .fetch_one(tx.acquire().await?)
        .await?;

        tx.commit().await?;

        Ok(puzzle)
    }
}
