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

use super::User;

#[derive(Debug, sqlx::FromRow)]
pub struct UltimateTicTacToe {
    id: i64,
    pub user_id: i64,
    pub opponent_id: i64,
    pub won: bool,
    created_at: chrono::NaiveDateTime,
}

impl UltimateTicTacToe {
    pub async fn create(user_id: i32, opponent_id: i32, won: bool, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
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

        Ok(tictactoe)
    }

    pub async fn get_standings(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(Vec<UTTTLeaderboardEntry>, bool)> {
        let (leaderboard, more) = UTTTLeaderboardEntryRaw::get_all_sorted(tx).await?;

        let mut entries = Vec::new();

        for entry in leaderboard {
            let user = User::get_by_id(entry.user_id, tx).await?.ok_or(anyhow::anyhow!("No user found"))?;

            let ratio = entry.ratio.ok_or(anyhow::anyhow!("No ratio found"))?;
            let wins = entry.total.unwrap_or(0);

            // ratio is wins - losses
            let losses = wins - ratio;

            // now that we've reconstructed the wins and losses, we can calculate the players rating.
            let rating = super::calculate_rating(wins, losses);

            entries.push(UTTTLeaderboardEntry { user, wins, losses, rating });
        }

        // sort entried by rating, highest to lowest
        entries.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));

        Ok((entries, more))
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UTTTLeaderboardEntryRaw {
    user_id: i64,
    ratio: Option<i64>,
    total: Option<i64>,
}

impl UTTTLeaderboardEntryRaw {
    async fn get_all_sorted(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(Vec<Self>, bool)> {
        let leaderboard = sqlx::query_as!(
            Self,
            r#"
            SELECT user_id, SUM((won::integer * 2) - 1) as ratio, SUM(won::integer) as total FROM ultimate_tictactoe GROUP BY user_id ORDER BY ratio DESC
            "#,
        )
        .fetch_all(tx.acquire().await?)
        .await?;

        Ok((leaderboard, false))
    }
}

pub struct UTTTLeaderboardEntry {
    pub user: User,
    pub wins: i64,
    pub losses: i64,
    pub rating: f64,
}
