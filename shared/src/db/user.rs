// CREATE TABLE IF NOT EXISTS users (
//     id SERIAL PRIMARY KEY,
//     name text NOT NULL, -- Will usually be the users global nickname, but can also be the users username. worst case scenario, it will be the users discord id
//     discord_id bigint NOT NULL, -- The users discord id
// );

use sqlx::Acquire;

use crate::anyhow::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub discord_id: i64,
}

impl User {
    pub async fn get_or_create(ctx: &serenity::client::Context, discord_id: &serenity::model::id::UserId, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
        let username = {
            let user = discord_id.to_user(&ctx.http).await?;
            match user.global_name {
                Some(name) => name,
                None => user.name,
            }
        };
        let user = match Self::get_by_discord_id(discord_id, tx).await? {
            Some(mut user) => {
                if user.name != username {
                    user.update_name(&username, tx).await?;
                }
                user
            }
            None => Self::create(ctx, discord_id, tx).await?,
        };
        Ok(user)
    }
    pub async fn get_by_discord_id(discord_id: &serenity::model::id::UserId, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Option<Self>> {
        let row = sqlx::query_as!(Self, "SELECT * FROM users WHERE discord_id = $1", discord_id.to_string().parse::<i64>()?)
            .fetch_optional(tx.acquire().await?)
            .await?;
        if let Some(row) = row {
            Ok(Some(Self {
                id: row.id,
                name: row.name,
                discord_id: row.discord_id,
            }))
        } else {
            Ok(None)
        }
    }
    pub async fn get_by_id(id: i64, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Option<Self>> {
        let row = sqlx::query_as!(Self, "SELECT * FROM users WHERE id = $1", id as i32).fetch_optional(tx.acquire().await?).await?;
        if let Some(row) = row {
            Ok(Some(Self {
                id: row.id,
                name: row.name,
                discord_id: row.discord_id,
            }))
        } else {
            Ok(None)
        }
    }
    pub async fn create(ctx: &serenity::client::Context, discord_id: &serenity::model::id::UserId, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Self> {
        let user = discord_id.to_user(&ctx.http).await?;

        let nickname = user.global_name.unwrap_or(user.name);

        let row = sqlx::query_as!(
            Self,
            "INSERT INTO users (name, discord_id) VALUES ($1, $2) RETURNING *",
            nickname,
            discord_id.to_string().parse::<i64>()?
        )
        .fetch_one(tx.acquire().await?)
        .await?;

        Ok(Self {
            id: row.id,
            name: row.name,
            discord_id: row.discord_id,
        })
    }
    async fn update_name(&mut self, name: &str, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<()> {
        sqlx::query!("UPDATE users SET name = $1 WHERE id = $2", name, self.id as i32).execute(tx.acquire().await?).await?;
        self.name = name.to_string();
        Ok(())
    }
}
