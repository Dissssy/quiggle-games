// CREATE TABLE IF NOT EXISTS ultimate_tictactoe (
//     -- multiplayer game, so we'll be keeping track of only if the player won or lost
//     id SERIAL PRIMARY KEY,
//     user_id integer NOT NULL REFERENCES users(id),
//     opponent_id integer NOT NULL REFERENCES users(id), -- not really used, we double up on the entries so its easier to query
//     won boolean NOT NULL, -- true if the user won, false if the user lost. we dont keep track of draws, draws are lame
//     created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
// );

use sqlx::{types::chrono, Acquire};

use crate::anyhow::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct UltimateTicTacToe {
    id: i64,
    pub user_id: i64,
    pub opponent_id: i64,
    pub won: bool,
    created_at: chrono::NaiveDateTime,
}

impl UltimateTicTacToe {
    pub async fn create(user_id: i32, opponent_id: i32, won: bool, db: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
        let mut tx = db.begin().await?;

        let tictactoe = sqlx::query_as!(
            UltimateTicTacToe,
            r#"
            INSERT INTO ultimate_tictactoe (user_id, opponent_id, won)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            user_id,
            opponent_id,
            won
        )
        .fetch_one(tx.acquire().await?)
        .await?;

        tx.commit().await?;

        Ok(tictactoe)
    }
}
