#![allow(dead_code)]

use base64::Engine;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::all::*;

pub use anyhow;
pub use async_recursion::async_recursion;
pub use async_trait::async_trait;
pub use colored;
pub use log;
pub use rand;
pub use serenity;
pub use sqlx;

pub mod db;

pub type OptTrans<'a> = Option<sqlx::Transaction<'a, sqlx::Postgres>>;

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
    fn register(&self) -> CreateCommand {
        let info = self.get_command_info();
        let mut b = CreateCommand::new(info.name);
        b = b.description(info.description);
        for option in info.options.0 {
            b = b.add_option(CreateCommandOption::new(option.option_type.into(), option.name, option.description).required(option.required));
        }
        b
    }
    fn get_command_info(&self) -> CommandInfo;
    #[allow(unused_variables)]
    async fn application_command(&mut self, ctx: &Context, interaction: &mut CommandInteraction, transaction: &mut OptTrans<'_>) -> Result<()> {
        log::error!("Interaction handler not implemented for {}", self.get_name().blue());

        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .content(format!("Command handler not implemented for `{}`", self.get_name())),
                ),
            )
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
    #[allow(unused_variables)]
    async fn message_component(&mut self, ctx: &Context, interaction: &mut ComponentInteraction, transaction: &mut OptTrans<'_>) -> Result<()> {
        log::error!("Message component handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("Message component handler not implemented for `{}`", self.get_name()))),
            )
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
    #[allow(unused_variables)]
    async fn autocomplete(&mut self, ctx: &Context, interaction: &mut CommandInteraction, transaction: &mut OptTrans<'_>) -> Result<()> {
        log::error!("Autocomplete handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Autocomplete(CreateAutocompleteResponse::new().add_string_choice(format!("Autocomplete handler not implemented for `{}`", self.get_name()), "epicfail")),
            )
            .await
        {
            log::error!("Error creating interaction response: {}", e);
        }
        Ok(())
    }
    #[allow(unused_variables)]
    async fn modal_submit(&mut self, ctx: &Context, interaction: &mut ModalInteraction, transaction: &mut OptTrans<'_>) -> Result<()> {
        log::error!("Modal submit handler not implemented for {}", self.get_name().blue());
        if let Err(e) = interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("Modal submit handler not implemented for `{}`", self.get_name()))),
            )
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

impl From<serenity::model::application::Command> for CommandInfo {
    fn from(command: serenity::model::application::Command) -> Self {
        Self {
            name: command.name,
            description: command.description,
            options: command.options.into_iter().map(|option| option.into()).collect::<Vec<CommandOption>>().into(),
        }
    }
}

impl CommandInfo {
    pub fn populate_subcommands(&mut self, command: serenity::all::Command) {
        for option in &mut self.options.0 {
            // find commandoption with matching name
            let command_option = command.options.iter().find(|o| o.name == option.name).unwrap();

            option.populate_subcommands(command_option);
        }
    }
}

#[derive(Debug)]
pub struct UnorderedVec<T>(pub Vec<T>);

impl<T> From<Vec<T>> for UnorderedVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> UnorderedVec<T> {
    pub fn push<N>(&mut self, val: N)
    where
        N: Into<T>,
    {
        self.0.push(val.into());
    }
}

impl<T> PartialEq for UnorderedVec<T>
where
    T: PartialEq + std::fmt::Debug,
{
    fn eq(&self, other: &Self) -> bool {
        let b = self.0.len() == other.0.len() && self.0.iter().all(|v| other.0.contains(v));
        if !b {
            println!("self: {:#?}", self.0);
            println!("other: {:#?}", other.0);
        }
        b
    }
}

impl<T> Eq for UnorderedVec<T> where T: Eq + std::fmt::Debug {}

// impl<T> IntoIterator for UnorderedVec<T> {
//     type Item = T;
//     type IntoIter = std::vec::IntoIter<T>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub option_type: CommandOptionType,
    pub choices: UnorderedVec<CommandOptionChoice>,
    pub required: bool,
}

impl From<serenity::model::application::CommandOption> for CommandOption {
    fn from(option: serenity::model::application::CommandOption) -> Self {
        Self {
            name: option.name,
            description: option.description,
            option_type: option.kind.into(),
            choices: option.choices.into_iter().map(|choice| choice.into()).collect::<Vec<CommandOptionChoice>>().into(),
            required: option.required,
        }
    }
}

impl CommandOption {
    pub fn populate_subcommands(&mut self, command_option: &serenity::model::application::CommandOption) {
        match &mut self.option_type {
            CommandOptionType::SubCommand(subcommands) => {
                for subcommand in &command_option.options {
                    let mut t: CommandOption = subcommand.clone().into();
                    t.populate_subcommands(subcommand);
                    subcommands.push(t);
                }

                // for subcommand in subcommands.0.iter_mut() {
                //     subcommand.populate_subcommands(command_option);
                // }
            }
            CommandOptionType::SubCommandGroup(subcommands) => {
                for subcommand in &command_option.options {
                    let mut t: CommandOption = subcommand.clone().into();
                    t.populate_subcommands(subcommand);
                    subcommands.push(t);
                }
            }
            _ => {}
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
    SubCommand(UnorderedVec<CommandOption>),
    SubCommandGroup(UnorderedVec<CommandOption>),
    Unknown(Option<u8>),
    User,
}

impl From<serenity::model::application::CommandOptionType> for CommandOptionType {
    fn from(option_type: serenity::model::application::CommandOptionType) -> Self {
        match option_type {
            serenity::model::application::CommandOptionType::Attachment => Self::Attachment,
            serenity::model::application::CommandOptionType::Boolean => Self::Boolean,
            serenity::model::application::CommandOptionType::Channel => Self::Channel,
            serenity::model::application::CommandOptionType::Integer => Self::Integer,
            serenity::model::application::CommandOptionType::Mentionable => Self::Mentionable,
            serenity::model::application::CommandOptionType::Number => Self::Number,
            serenity::model::application::CommandOptionType::Role => Self::Role,
            serenity::model::application::CommandOptionType::String => Self::String,
            serenity::model::application::CommandOptionType::SubCommand => Self::SubCommand(UnorderedVec::from(Vec::new())),
            serenity::model::application::CommandOptionType::SubCommandGroup => Self::SubCommandGroup(UnorderedVec::from(Vec::new())),
            serenity::model::application::CommandOptionType::Unknown(i) => Self::Unknown(Some(i)),
            serenity::model::application::CommandOptionType::User => Self::User,
            _ => Self::Unknown(None),
        }
    }
}

impl From<CommandOptionType> for serenity::model::application::CommandOptionType {
    fn from(val: CommandOptionType) -> Self {
        match val {
            CommandOptionType::Attachment => serenity::model::application::CommandOptionType::Attachment,
            CommandOptionType::Boolean => serenity::model::application::CommandOptionType::Boolean,
            CommandOptionType::Channel => serenity::model::application::CommandOptionType::Channel,
            CommandOptionType::Integer => serenity::model::application::CommandOptionType::Integer,
            CommandOptionType::Mentionable => serenity::model::application::CommandOptionType::Mentionable,
            CommandOptionType::Number => serenity::model::application::CommandOptionType::Number,
            CommandOptionType::Role => serenity::model::application::CommandOptionType::Role,
            CommandOptionType::String => serenity::model::application::CommandOptionType::String,
            CommandOptionType::SubCommand(_) => serenity::model::application::CommandOptionType::SubCommand,
            CommandOptionType::SubCommandGroup(_) => serenity::model::application::CommandOptionType::SubCommandGroup,
            CommandOptionType::Unknown(i) => serenity::model::application::CommandOptionType::Unknown(i.unwrap_or(0)),
            CommandOptionType::User => serenity::model::application::CommandOptionType::User,
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

pub fn format_duration(t: u64) -> String {
    let mut t = t;
    let mut s = String::new();
    if t >= 86400 {
        let d = t / 86400;
        t %= 86400;
        s.push_str(&format!("{}d ", d));
    }
    if t >= 3600 {
        let h = t / 3600;
        t %= 3600;
        s.push_str(&format!("{}h ", h));
    }
    if t >= 60 {
        let m = t / 60;
        t %= 60;
        s.push_str(&format!("{}m ", m));
    }
    if t > 0 {
        s.push_str(&format!("{}s", t));
    } else if s.is_empty() {
        s.push_str("0s");
    }
    s.trim().to_string()
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandOptionChoice {
    pub name: String,
    pub value: String,
}

impl From<serenity::model::application::CommandOptionChoice> for CommandOptionChoice {
    fn from(choice: serenity::model::application::CommandOptionChoice) -> Self {
        let value = choice.value.to_string();

        // attempt to strip quotes
        let value = if value.starts_with('"') && value.ends_with('"') {
            value[1..value.len() - 1].to_string()
        } else {
            value
        };

        Self { name: choice.name, value }
    }
}
