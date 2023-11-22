#![allow(dead_code)]

use qg_shared::{
    anyhow::{anyhow, Result},
    colored::*,
    log,
};
use serenity::{
    client::Context,
    client::EventHandler,
    futures::lock::Mutex,
    model::application::interaction::Interaction,
    model::id::{CommandId, GuildId},
};
use std::{collections::HashMap, sync::Arc};

pub struct Handler {
    commands: Arc<Mutex<CommandHolder>>,
}

impl Handler {
    pub fn new(dev_server: Option<GuildId>) -> Self {
        Self {
            commands: Arc::new(Mutex::new(CommandHolder::new(dev_server))),
        }
    }
    pub async fn register_commands(&self, http: &Arc<serenity::http::Http>) -> Result<()> {
        let mut commands = self.commands.lock().await;
        #[cfg(feature = "ping")]
        commands.register(http, Arc::new(Mutex::new(qg_ping::command()))).await?;
        commands.register(http, Arc::new(Mutex::new(qg_tictactoe::command()))).await?;
        commands.register(http, Arc::new(Mutex::new(qg_ulttictactoe::command()))).await?;
        commands.register(http, Arc::new(Mutex::new(qg_slidingpuzzle::command()))).await?;
        commands.finalize_registration(http).await?;
        Ok(())
    }
}

#[qg_shared::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::client::Context, ready: serenity::model::gateway::Ready) {
        // eventually we will want to register all of our commands here
        if let Err(e) = self.register_commands(&ctx.http).await {
            log::error!("Error registering commands: {}", e);
        }
        log::info!("{} is connected!", ready.user.name);
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Ping(p) => {
                log::info!("Ping interaction {}", format!("{:?}", p).blue());
            }
            Interaction::ApplicationCommand(mut cmd) => {
                let name = cmd.data.name.clone();
                if let Some(command) = {
                    let commands = self.commands.lock().await;
                    commands.find(|c| c == name)
                } {
                    if let Err(e) = command.lock().await.application_command(&ctx, &mut cmd).await {
                        log::trace!("Error handling interaction for command {}: {}", name.blue(), e.to_string().red());
                        if let Err(e) = cmd.create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(e).ephemeral(true))).await {
                            log::error!("Error creating interaction response: {}", e);
                        }
                    }
                } else {
                    log::warn!("Command {} not found", name.red());
                    if let Err(e) = cmd
                        .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Command `{}` not found", name)).ephemeral(true)))
                        .await
                    {
                        log::error!("Error creating interaction response: {}", e);
                    }
                }
            }
            Interaction::MessageComponent(mut cmp) => {
                log::trace!("Message component interaction {}", format!("{:?}", cmp).blue());
                let name = cmp.data.custom_id.clone();
                log::trace!("Message component interaction {}", name.blue());
                if let Some(command) = {
                    log::trace!("locking commands");
                    let commands = self.commands.lock().await;
                    log::trace!("finding command");
                    commands.find(|c| name.starts_with(c))
                } {
                    let mut cmd = command.lock().await;
                    log::trace!("found command: {}", cmd.get_name().blue());
                    if let Err(e) = cmd.message_component(&ctx, &mut cmp).await {
                        log::trace!("Error handling interaction for command {}: {}", name.blue(), e.to_string().red());
                        if let Err(e) = cmp.create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(e).ephemeral(true))).await {
                            log::error!("Error creating interaction response: {}", e);
                        }
                    } else {
                        log::trace!("Handled interaction for command {}", name.blue());
                        if let Err(e) = cmp
                            .create_interaction_response(&ctx.http, |f| f.kind(serenity::model::application::interaction::InteractionResponseType::DeferredUpdateMessage))
                            .await
                        {
                            log::trace!("Error creating interaction response: {}", e);
                        }
                    }
                } else {
                    log::warn!("Command {} not found", name.red());
                    if let Err(e) = cmp
                        .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Command `{}` not found", name)).ephemeral(true)))
                        .await
                    {
                        log::error!("Error creating interaction response: {}", e);
                    }
                }
            }
            Interaction::Autocomplete(mut act) => {
                log::info!("Autocomplete interaction {}", format!("{:?}", act).blue());
                let name = act.data.name.clone();
                if let Some(command) = {
                    let commands = self.commands.lock().await;
                    commands.find(|c| c == name)
                } {
                    if let Err(e) = command.lock().await.autocomplete(&ctx, &mut act).await {
                        log::trace!("Error handling interaction for command {}: {}", name.blue(), e.to_string().red());
                        if let Err(e) = act.create_autocomplete_response(&ctx.http, |f| f.add_string_choice(e, "epicfail")).await {
                            log::error!("Error creating interaction response: {}", e);
                        }
                    }
                } else {
                    log::warn!("Command {} not found", name.red());
                    if let Err(e) = act
                        .create_autocomplete_response(&ctx.http, |f| f.add_string_choice(format!("Command {} not found", name), "epicfail"))
                        .await
                    {
                        log::error!("Error creating interaction response: {}", e);
                    }
                }
            }
            Interaction::ModalSubmit(mut mdl) => {
                log::info!("Modal submit interaction {}", format!("{:?}", mdl).blue());
                let name = mdl.data.custom_id.clone();
                if let Some(command) = {
                    let commands = self.commands.lock().await;
                    commands.find(|c| name.starts_with(c))
                } {
                    if let Err(e) = command.lock().await.modal_submit(&ctx, &mut mdl).await {
                        log::trace!("Error handling interaction for command {}: {}", name.blue(), e.to_string().red());
                        if let Err(e) = mdl.create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(e).ephemeral(true))).await {
                            log::error!("Error creating interaction response: {}", e);
                        }
                    }
                } else {
                    log::warn!("Command {} not found", name.red());
                    if let Err(e) = mdl
                        .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content(format!("Command `{}` not found", name)).ephemeral(true)))
                        .await
                    {
                        log::error!("Error creating interaction response: {}", e);
                    }
                }
            }
        }
    }
}

pub struct CommandHolder {
    cached_commands: Option<Vec<(CommandId, qg_shared::CommandInfo)>>,
    commands: HashMap<String, Arc<Mutex<dyn qg_shared::Command>>>,
    dev_server: Option<GuildId>,
}

impl CommandHolder {
    pub fn new(dev_server: Option<GuildId>) -> Self {
        Self {
            commands: HashMap::new(),
            cached_commands: None,
            dev_server,
        }
    }

    pub fn find(&self, predicate: impl Fn(&str) -> bool) -> Option<Arc<Mutex<dyn qg_shared::Command>>> {
        self.commands.iter().find(|(name, _)| predicate(name)).map(|(_, command)| command.clone())
    }

    pub async fn register(&mut self, http: &Arc<serenity::http::Http>, raw_command: Arc<Mutex<dyn qg_shared::Command>>) -> Result<()> {
        // get and compare the command info to see if it needs registered/updated
        // attempt to get the cached command info from self.cached_commands, if it is None, call self.cache_commands, then try again
        let cached_commands = match &self.cached_commands {
            Some(cached_commands) => cached_commands,
            None => {
                self.cache_commands(http).await?;
                self.cached_commands.as_ref().ok_or(anyhow!("self.cached_commands was None after calling self.cache_commands"))?
            }
        };

        let name = {
            let command = raw_command.lock().await;
            let command_info = command.get_command_info();
            // ensure there is no command with the same name already registered
            let name = command_info.name.clone();
            if self.commands.contains_key(&name) {
                return Err(anyhow!("Command with name {} already registered", name.red()));
            }
            if cached_commands
                .iter()
                .find(|(_, cached_command)| cached_command.name == command_info.name)
                .map(|(_, cached_command)| cached_command != &command_info)
                .unwrap_or(true)
            {
                // since the command is NOT registered OR the command info is different, register the command
                if let Some(dev_server) = self.dev_server {
                    log::info!("Registering command {} to {}", command_info.name.blue(), "DEV SERVER".red().bold());
                    let guild = http.get_guild(dev_server.0).await?;
                    guild.create_application_command(http, |b| command.register(b)).await?;
                } else {
                    log::info!("Registering command {} {}", command_info.name.blue(), "GLOBALLY".green().bold());
                    serenity::model::application::command::Command::create_global_application_command(http, |b| command.register(b)).await?;
                }
            }
            name
        };

        // insert the command into self.commands
        self.commands.insert(name, raw_command);
        Ok(())
    }

    async fn cache_commands(&mut self, http: &Arc<serenity::http::Http>) -> Result<()> {
        // get the commands from discord and cache them in self.cached_commands
        self.cached_commands = Some(match self.dev_server {
            Some(dev_server) => {
                log::info!("Caching dev commands");
                http.get_guild(dev_server.0)
                    .await?
                    .get_application_commands(http)
                    .await?
                    .into_iter()
                    .map(|command| (command.id, command.into()))
                    .collect()
            }
            None => {
                log::info!("Caching global commands");
                http.get_global_application_commands().await?.into_iter().map(|command| (command.id, command.into())).collect()
            }
        });
        Ok(())
    }

    pub async fn finalize_registration(&mut self, http: &Arc<serenity::http::Http>) -> Result<()> {
        // find any commands that are registered but not in self.commands and unregister them
        let cached_commands = match &self.cached_commands {
            Some(cached_commands) => cached_commands,
            None => {
                self.cache_commands(http).await?;
                self.cached_commands.as_ref().ok_or(anyhow!("self.cached_commands was None after calling self.cache_commands"))?
            }
        };

        let dev_guild = match self.dev_server {
            Some(dev_guild) => Some(http.get_guild(dev_guild.0).await?),
            None => None,
        };

        for (id, cached_command) in cached_commands {
            if !self.commands.keys().any(|name| name == &cached_command.name) {
                match dev_guild.as_ref() {
                    Some(dev_guild) => {
                        log::info!("Unregistering command {} from {}", cached_command.name.blue(), "DEV SERVER".red().bold());
                        dev_guild.delete_application_command(http, *id).await?;
                    }
                    None => {
                        log::info!("Unregistering command {} {}", cached_command.name.blue(), "GLOBALLY".green().bold());
                        serenity::model::application::command::Command::delete_global_application_command(http, *id).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
