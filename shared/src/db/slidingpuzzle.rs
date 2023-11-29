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

use serenity::futures::TryStreamExt;
use sqlx::{types::chrono, Acquire};

use crate::anyhow::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct SlidingPuzzle {
    id: i64,
    user_id: i64,
    pub difficulty: i32,
    pub size: i32,
    pub score: i32,
    pub time: i32,
    created_at: chrono::NaiveDateTime,
}

impl SlidingPuzzle {
    pub async fn create(user_id: i32, difficulty: i32, size: i32, score: i32, time: i32, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
        let puzzle = sqlx::query_as!(
            Self,
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

        Ok(puzzle)
    }

    pub async fn get_standings(filters: SlidingPuzzleFilters, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(Vec<SlidingPuzzleWithUser>, bool)> {
        let order = match filters.sort_by {
            SlidingPuzzleSortBy::Score => "score",
            SlidingPuzzleSortBy::Time => "time",
        };

        let mut puzzles = Vec::new();

        {
            let mut rows = sqlx::query_as!(
                Self,
                r#"
                SELECT * FROM slidingpuzzle
                WHERE size = $1
                and difficulty = $2
                ORDER BY $3 ASC
                "#,
                filters.filter_by.size,
                filters.filter_by.difficulty,
                order
            )
            .fetch(tx.acquire().await?);

            let mut users = std::collections::HashMap::new();

            while let Some::<SlidingPuzzle>(puzzle) = rows.try_next().await? {
                let score = puzzle.score;
                // if user is in hashmap, check if the score is lower than the current score
                // if it is, replace the score with the new score and update the puzzle in the puzzles vector, one per user
                if let Some(otherscore) = users.get_mut(&puzzle.user_id) {
                    if score < *otherscore {
                        *otherscore = score;
                        puzzles.retain(|p: &SlidingPuzzle| p.user_id != puzzle.user_id);
                        puzzles.push(puzzle);
                    }
                } else {
                    users.insert(puzzle.user_id, score);
                    puzzles.push(puzzle);
                }
            }
            match filters.sort_by {
                SlidingPuzzleSortBy::Score => puzzles.sort_by(|a, b| {
                    // sort by score, if score is equal, sort by time
                    // if time is equal sort by date
                    match (a.score == b.score, a.time == b.time) {
                        (true, true) => a.created_at.cmp(&b.created_at),
                        (true, _) => a.time.cmp(&b.time),
                        (_, _) => a.score.cmp(&b.score),
                    }
                }),
                SlidingPuzzleSortBy::Time => puzzles.sort_by(|a, b| {
                    // sort by time, if time is equal, sort by score
                    // if score is equal sort by date
                    match (a.time == b.time, a.score == b.score) {
                        (true, true) => a.created_at.cmp(&b.created_at),
                        (true, _) => a.score.cmp(&b.score),
                        (_, _) => a.time.cmp(&b.time),
                    }
                }),
            }
        }

        let more_available = puzzles.len() > filters.limit as usize;

        if more_available {
            puzzles.truncate(filters.limit as usize);
        }

        let mut results = Vec::new();

        for puzzle in puzzles {
            let user = super::User::get_by_id(puzzle.user_id, tx).await?.ok_or(anyhow::anyhow!("User not found"))?;
            results.push(SlidingPuzzleWithUser { puzzle, user });
        }

        Ok((results, more_available))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SlidingPuzzleFilters {
    sort_by: SlidingPuzzleSortBy,
    filter_by: SlidingPuzzleFilterBy,
    limit: i64,
    offset: i64, // will be multiplied by limit, pagination
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub enum SlidingPuzzleSortBy {
    Score,
    Time,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SlidingPuzzleFilterBy {
    pub difficulty: i32,
    pub size: i32,
}

impl SlidingPuzzleFilters {
    pub fn sort_by_score(&mut self) -> &mut Self {
        self.sort_by = SlidingPuzzleSortBy::Score;
        self
    }
    pub fn sort_by_time(&mut self) -> &mut Self {
        self.sort_by = SlidingPuzzleSortBy::Time;
        self
    }
    pub fn easy(&mut self) -> &mut Self {
        self.filter_by.difficulty = 0;
        self
    }
    pub fn medium(&mut self) -> &mut Self {
        self.filter_by.difficulty = 1;
        self
    }
    pub fn hard(&mut self) -> &mut Self {
        self.filter_by.difficulty = 2;
        self
    }
    pub fn threebythree(&mut self) -> &mut Self {
        self.filter_by.size = 0;
        self
    }
    pub fn fourbyfour(&mut self) -> &mut Self {
        self.filter_by.size = 1;
        self
    }
    pub fn fivebyfive(&mut self) -> &mut Self {
        self.filter_by.size = 2;
        self
    }
    pub fn limit(&mut self, limit: i64) -> &mut Self {
        self.limit = limit;
        self
    }
    pub fn set_offset(&mut self, offset: i64) -> &mut Self {
        self.offset = offset;
        self
    }
    pub fn increment_offset(&mut self) -> &mut Self {
        self.offset += 1;
        self
    }
    pub fn decrement_offset(&mut self) -> &mut Self {
        self.offset -= 1;
        self
    }
    fn paginated_offset(&self) -> i64 {
        self.offset * self.limit
    }
}

impl Default for SlidingPuzzleFilters {
    fn default() -> Self {
        Self {
            sort_by: SlidingPuzzleSortBy::Score,
            filter_by: SlidingPuzzleFilterBy { difficulty: 0, size: 0 },
            limit: 10,
            offset: 0,
        }
    }
}

pub struct SlidingPuzzleWithUser {
    pub puzzle: SlidingPuzzle,
    pub user: super::User,
}
