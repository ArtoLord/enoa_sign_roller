use anyhow::Result;
use log::info;
use serenity::all::{CacheHttp, CommandInteraction, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};

use crate::{commands::utils, discord::Handler};

pub async fn run(handler: &Handler, _ctx: impl CacheHttp, interaction: &CommandInteraction) -> Result<CreateInteractionResponse> {
    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id;

    if guild_id.is_none() {
        return Ok(
            utils::format_error("Мне можно написать только с сервера.")
        );
    }

    let guild_id = guild_id.unwrap();
    info!("Rolling sign for user {} from guild {}", user_id, guild_id);

    let user = handler.dao().get_user_info(user_id.get(), guild_id.get()).await?;

    let power = user.map_or(10, |u| u.shaman_power);

    let msg = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(format!("Твоя сила шамана: {}", power))
            .ephemeral(true)
    );

    Ok(msg)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("sign_my_power").description("Show my current shaman power")
}