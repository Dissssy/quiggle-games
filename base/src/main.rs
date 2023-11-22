use qg_shared::anyhow::anyhow;
mod custom_serenity;
use custom_serenity as shuttle_serenity;
use serenity::{model::gateway::GatewayIntents, Client};

mod handler;

#[allow(unused_variables)]
#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = &std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    )]
    db: sqlx::PgPool,
) -> shuttle_serenity::ShuttleSerenity {
    #[cfg(feature = "leaderboard")]
    {
        // if leaderboard is enabled, we need to make sure the database is set up
        sqlx::migrate!("../migrations").run(&db).await.map_err(|e| anyhow!(e))?;
    }

    let token = match secret_store.get("DISCORD_TOKEN") {
        Some(token) => token,
        None => {
            qg_shared::log::error!("DISCORD_TOKEN not found in secret store");
            return Err(anyhow!("DISCORD_TOKEN not found in secret store").into());
        }
    };

    if let Some(b) = secret_store.get("ALLOW_SELF_PLAY").and_then(|f| f.parse::<bool>().ok()) {
        qg_shared::log::info!("ALLOW_SELF_PLAY set to {}", b);
        std::env::set_var("ALLOW_SELF_PLAY", b.to_string());
    }

    let dev_server = secret_store.get("DEV_SERVER").and_then(|f| f.parse::<serenity::model::id::GuildId>().ok());

    // replace with actually necessary intents eventually lol
    let intents = GatewayIntents::non_privileged();

    let handler = {
        #[cfg(feature = "leaderboard")]
        {
            handler::Handler::new(dev_server, db)
        }
        #[cfg(not(feature = "leaderboard"))]
        {
            handler::Handler::new(dev_server)
        }
    };

    let client = Client::builder(&token, intents).event_handler(handler).await.map_err(|e| anyhow!(e))?;

    Ok(client.into())
}
