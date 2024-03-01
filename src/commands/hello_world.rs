use anyhow::Result;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<CreateInteractionResponse> {
    Ok(CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("Hello")
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("hello").description("Says hello")
}