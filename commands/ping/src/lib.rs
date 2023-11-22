use qg_shared::{anyhow::Result, serenity::all::*};

pub fn command() -> PingCommand {
    PingCommand
}

pub struct PingCommand;

#[qg_shared::async_trait]
impl qg_shared::Command for PingCommand {
    // fn register<'a>(&self, builder: &'a mut CreateApplicationCommand) -> &'a mut CreateApplicationCommand {
    //     let info = self.get_command_info();
    //     builder.name(info.name).description(info.description);
    //     builder
    // }

    fn get_command_info(&self) -> qg_shared::CommandInfo {
        qg_shared::CommandInfo {
            name: String::from("ping"),
            description: String::from("Ping the bot"),
            options: Vec::new().into(),
        }
    }

    async fn application_command(&mut self, ctx: &Context, interaction: &mut CommandInteraction, _: &mut qg_shared::OptTrans<'_>) -> Result<()> {
        interaction
            .create_response(&ctx.http, {
                CreateInteractionResponse::Message(CreateInteractionResponseMessage::default().content("Pong!").ephemeral(true))
            })
            .await?;
        Ok(())
    }
}
