use anyhow::Result;
use log::info;
use rand::Rng;
use serenity::all::{CommandInteraction, Context, CreateButton, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage};

use crate::{commands::utils, discord::Handler, signs};

pub async fn run(handler: &Handler, _ctx: &Context, interaction: &CommandInteraction) -> Result<Option<CreateInteractionResponse>> {
    let user_id = interaction.user.id;
    let guild_id = interaction.guild_id;

    if guild_id.is_none() {
        return Ok(Some(
            utils::format_error("Мне можно написать только с сервера.")
        ));
    }

    let guild_id = guild_id.unwrap();
    info!("Rolling sign for user {} from guild {}", user_id, guild_id);

    let mut rand_seq = vec![];

    for _ in 1..=4 {
        rand_seq.push(rand::thread_rng().gen_range(1..=4).to_string());
    }

    rand_seq.sort();

    info!("Generated sequance for user {} form guild {} is {:?}", user_id, guild_id, rand_seq);

    let sign_id = rand_seq.join("");

    let dao = handler.dao();
    let guild = dao.create_sign(guild_id.get(), sign_id.clone(), user_id.get()).await?;

    if guild.is_none() {
        info!("Sign for guild {} already exists, skipping request from user {}", guild_id, user_id);
        return Ok(Some(
            utils::format_error(
                "Знамение на сегодня уже создано, приходи завтра"
            ))
        );
    }

    let guild = guild.unwrap();

    let msg = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(signs::render_sign(guild.current_sign))
            .button(
                CreateButton::new("change_sign")
                    .style(serenity::all::ButtonStyle::Primary)
                    .label("Повлиять на знамение")
                )
    );

    Ok(Some(msg))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("sign_roll").description("Roll enoa sign")
}