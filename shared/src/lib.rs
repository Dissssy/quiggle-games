#![allow(dead_code)]
use serenity::{builder::CreateApplicationCommand, client::Context, model::application::interaction::application_command::ApplicationCommandInteraction};

pub use anyhow;
pub use async_trait::async_trait;
pub use colored;
pub use serenity;

use colored::*;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Command
where
    Self: Send + Sync,
{
    fn get_name(&self) -> String {
        self.get_command_info().name
    }
    fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand;
    fn get_command_info(&self) -> CommandInfo;
    async fn application_command(&mut self, ctx: Context, interaction: ApplicationCommandInteraction) -> Result<()> {
        log::error!("Interaction handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_interaction_response(&ctx.http, |f| {
                f.interaction_response_data(|d| d.content(format!("Interaction handler not implemented for `{}`", self.get_name())).ephemeral(true))
            })
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
}

impl From<serenity::model::application::command::Command> for CommandInfo {
    fn from(command: serenity::model::application::command::Command) -> Self {
        Self {
            name: command.name,
            description: command.description,
            options: command.options.into_iter().map(|option| option.into()).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub required: bool,
}

impl From<serenity::model::application::command::CommandOption> for CommandOption {
    fn from(option: serenity::model::application::command::CommandOption) -> Self {
        Self {
            name: option.name,
            description: option.description,
            required: option.required,
        }
    }
}
