use qg_shared::{
    anyhow::Result,
    serenity::{builder::CreateApplicationCommand, client::Context, model::application::interaction::application_command::ApplicationCommandInteraction},
};

pub fn command() -> PingCommand {
    PingCommand
}

pub struct PingCommand;

#[qg_shared::async_trait]
impl qg_shared::Command for PingCommand {
    fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand {
        let info = self.get_command_info();
        builder.name(info.name).description(info.description);
        builder
    }

    fn get_command_info(&self) -> qg_shared::CommandInfo {
        qg_shared::CommandInfo {
            name: String::from("ping"),
            description: String::from("Ping the bot"),
            options: Vec::new(),
        }
    }

    async fn application_command(&mut self, ctx: Context, interaction: ApplicationCommandInteraction) -> Result<()> {
        interaction
            .create_interaction_response(&ctx.http, |f| f.interaction_response_data(|d| d.content("Pong!").ephemeral(true)))
            .await?;
        Ok(())
    }
}
