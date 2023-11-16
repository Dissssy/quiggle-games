use qg_shared::anyhow::anyhow;
use serenity::{model::gateway::GatewayIntents, Client};

mod handler;

#[shuttle_runtime::main]
async fn serenity(#[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore) -> shuttle_serenity::ShuttleSerenity {
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

    let handler = handler::Handler::new(dev_server);

    let client = Client::builder(&token, intents).event_handler(handler).await.map_err(|e| anyhow!(e))?;

    Ok(client.into())
}
