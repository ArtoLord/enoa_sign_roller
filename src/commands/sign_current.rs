use anyhow::Result;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};

use crate::{commands::utils, discord::Handler, signs};

pub async fn run(handler: &Handler, ctx: &Context, interaction: &CommandInteraction) -> Result<CreateInteractionResponse> {
    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id;

    if guild_id.is_none() {
        return Ok(
            utils::format_error("Мне можно написать только с сервера.")
        );
    }
    let guild_id = guild_id.unwrap();

    let guild_info = handler.dao().get_guild_info(guild_id.get()).await?;

    if guild_info.is_none() {
        return Ok(CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(format!("Сегодня еще не было знамения. Ты можешь создать его!"))
                .ephemeral(true)
        ))
    }

    let guild_info = guild_info.unwrap();
    Ok(CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(signs::render_sign(guild_info.current_sign))
            .ephemeral(true)
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("sign_current").description("Get current enoa sign for this guild")
}