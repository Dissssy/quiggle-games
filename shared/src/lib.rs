#![allow(dead_code)]

use base64::Engine;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, autocomplete::AutocompleteInteraction, message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
    },
};

pub use anyhow;
pub use async_trait::async_trait;
pub use colored;
pub use log;
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
    async fn application_command(&mut self, ctx: &Context, interaction: &mut ApplicationCommandInteraction) -> Result<()> {
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
    async fn message_component(&mut self, ctx: &Context, interaction: &mut MessageComponentInteraction) -> Result<()> {
        log::error!("Message component handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_interaction_response(&ctx.http, |f| {
                f.interaction_response_data(|d| d.content(format!("Message component handler not implemented for `{}`", self.get_name())).ephemeral(true))
            })
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
    async fn autocomplete(&mut self, ctx: &Context, interaction: &mut AutocompleteInteraction) -> Result<()> {
        log::error!("Autocomplete handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_autocomplete_response(&ctx.http, |f| {
                f.add_string_choice(format!("Autocomplete handler not implemented for `{}`", self.get_name()), "epicfail")
            })
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
    async fn modal_submit(&mut self, ctx: &Context, interaction: &mut ModalSubmitInteraction) -> Result<()> {
        log::error!("Modal submit handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_interaction_response(&ctx.http, |f| {
                f.interaction_response_data(|d| d.content(format!("Modal submit handler not implemented for `{}`", self.get_name())).ephemeral(true))
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
    pub options: UnorderedVec<CommandOption>,
}

impl From<serenity::model::application::command::Command> for CommandInfo {
    fn from(command: serenity::model::application::command::Command) -> Self {
        Self {
            name: command.name,
            description: command.description,
            options: command.options.into_iter().map(|option| option.into()).collect::<Vec<CommandOption>>().into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnorderedVec<T>(Vec<T>);

impl<T> From<Vec<T>> for UnorderedVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> IntoIterator for UnorderedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub option_type: CommandOptionType,
    pub required: bool,
}

impl From<serenity::model::application::command::CommandOption> for CommandOption {
    fn from(option: serenity::model::application::command::CommandOption) -> Self {
        Self {
            name: option.name,
            description: option.description,
            option_type: option.kind.into(),
            required: option.required,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandOptionType {
    Attachment,
    Boolean,
    Channel,
    Integer,
    Mentionable,
    Number,
    Role,
    String,
    SubCommand,
    SubCommandGroup,
    Unknown,
    User,
}

impl From<serenity::model::application::command::CommandOptionType> for CommandOptionType {
    fn from(option_type: serenity::model::application::command::CommandOptionType) -> Self {
        match option_type {
            serenity::model::application::command::CommandOptionType::Attachment => Self::Attachment,
            serenity::model::application::command::CommandOptionType::Boolean => Self::Boolean,
            serenity::model::application::command::CommandOptionType::Channel => Self::Channel,
            serenity::model::application::command::CommandOptionType::Integer => Self::Integer,
            serenity::model::application::command::CommandOptionType::Mentionable => Self::Mentionable,
            serenity::model::application::command::CommandOptionType::Number => Self::Number,
            serenity::model::application::command::CommandOptionType::Role => Self::Role,
            serenity::model::application::command::CommandOptionType::String => Self::String,
            serenity::model::application::command::CommandOptionType::SubCommand => Self::SubCommand,
            serenity::model::application::command::CommandOptionType::SubCommandGroup => Self::SubCommandGroup,
            serenity::model::application::command::CommandOptionType::Unknown => Self::Unknown,
            serenity::model::application::command::CommandOptionType::User => Self::User,
            _ => Self::Unknown,
        }
    }
}

impl From<CommandOptionType> for serenity::model::application::command::CommandOptionType {
    fn from(val: CommandOptionType) -> Self {
        match val {
            CommandOptionType::Attachment => serenity::model::application::command::CommandOptionType::Attachment,
            CommandOptionType::Boolean => serenity::model::application::command::CommandOptionType::Boolean,
            CommandOptionType::Channel => serenity::model::application::command::CommandOptionType::Channel,
            CommandOptionType::Integer => serenity::model::application::command::CommandOptionType::Integer,
            CommandOptionType::Mentionable => serenity::model::application::command::CommandOptionType::Mentionable,
            CommandOptionType::Number => serenity::model::application::command::CommandOptionType::Number,
            CommandOptionType::Role => serenity::model::application::command::CommandOptionType::Role,
            CommandOptionType::String => serenity::model::application::command::CommandOptionType::String,
            CommandOptionType::SubCommand => serenity::model::application::command::CommandOptionType::SubCommand,
            CommandOptionType::SubCommandGroup => serenity::model::application::command::CommandOptionType::SubCommandGroup,
            CommandOptionType::Unknown => serenity::model::application::command::CommandOptionType::Unknown,
            CommandOptionType::User => serenity::model::application::command::CommandOptionType::User,
        }
    }
}

pub fn serialize(data: &impl serde::Serialize) -> Result<String> {
    // let data = serde_json::to_string(data)?.into_bytes();
    let data = rmp_serde::encode::to_vec(data)?;
    let data = {
        let mut d = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut d, &data)?;
        d.finish()?
    };
    let data = { base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data) };
    Ok(data)
}

pub fn deserialize<T>(data: &'_ str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data)?;
    let data = {
        let mut d = flate2::read::GzDecoder::new(&data[..]);
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut d, &mut data)?;
        data
    };
    Ok(match rmp_serde::decode::from_slice::<T>(&data) {
        Ok(data) => data,
        Err(e) => {
            // since we used to use serde, we're gonna try that for backwards compatibility
            serde_json::from_slice::<T>(&data).map_err(|_| e)? // if this fails, we'll return the original error
        }
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CycleVec<T> {
    vec: Vec<T>,
    index: usize,
}

impl<T> CycleVec<T> {
    pub fn new(vec: Vec<T>) -> Self {
        // shuffle
        let mut vec = vec;
        vec.shuffle(&mut rand::thread_rng());
        Self { vec, index: 0 }
    }
    pub fn next_player(&mut self) {
        self.index = (self.index + 1) % self.vec.len();
    }
    pub fn current(&self) -> Option<&T> {
        self.vec.get(self.index)
    }
    pub fn all(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }
}

pub fn current_time() -> Result<u64> {
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    Ok(time)
}
