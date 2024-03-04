use anyhow::Result;
use indoc::formatdoc;
use log::info;
use rand::Rng;
use serenity::all::{ComponentInteraction, Context, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage};

use crate::{commands::utils, db::{SignState, UserInfo}, discord::Handler, signs::{self, render_sign}};

pub async fn run(handler: &Handler, ctx: &Context, interaction: &mut ComponentInteraction) -> Result<Option<CreateInteractionResponse>> {
    let user_id = interaction.user.id.get();
    let guild_id = interaction.guild_id;

    if guild_id.is_none() {
        return Ok(Some(
            utils::format_error("Мне можно написать только с сервера.")
        ));
    }

    let guild_id = guild_id.unwrap().get();
    info!("Changing sign by user {} from guild {}", user_id, guild_id);
    let dao = handler.dao();

    let user_info = dao.get_user_info(user_id, guild_id).await?;
    let guild_info = dao.get_guild_info(guild_id).await?;

    if guild_info.is_none() {
        return Ok(Some(utils::format_error("Сегодня еще не было знамения. Ты можешь его создать!")));
    }
    let guild_info = guild_info.unwrap();

    let mut user_info = match user_info {
        Some(u) => u,
        None => UserInfo { id: user_id, guild_id: guild_id, shaman_power: 10 }
    };

    let m = user_info.shaman_power / 2 - 5;
    let roll = rand::thread_rng().gen_range(1..=20);
    let value = roll + m;
    let difficulty = signs::get_difficulty(guild_info.current_sign.id);
    let mut shaman_power_decreased = false;
    let mut success = false;

    let state = if value >= difficulty {
        if rand::thread_rng().gen_bool(0.5) {
            shaman_power_decreased = true;
            user_info.shaman_power -= 1;
        }
        success = true;
        SignState::Success { by_user_id: user_id }
    } else {
        user_info.shaman_power += 1;
        SignState::Failed { by_user_id: user_id }
    };

    let res = dao.change_sign_state(guild_id, state).await?;
    if res.is_err() {
        let res = res.err().unwrap();
        if res.is_none() {
            return Ok(Some(utils::format_error("Сегодня еще не было знамения. Ты можешь его создать!")));
        }

        let res = res.unwrap();
        if res.current_sign.state != SignState::Created {
            return Ok(Some(utils::format_error("Кто-то уже повлиял на знамение сегодня")));
        }

        if res.current_sign.created_by_user_id == user_id {
            return Ok(Some(utils::format_error("Повлиять на знамение может только тот, кто его не создавал")));
        }

        return Ok(Some(utils::format_error("Ты не можешь повлиять на знамение сейчас")));
    }

    dao.save_user_info(user_info.clone()).await?;
    let res = res.ok().unwrap();
    let content = interaction.message.content.clone();

    interaction.message.edit(ctx, EditMessage::new()
        .content(content)
        .button(CreateButton::new("change_sign")
            .disabled(true)
            .style(serenity::all::ButtonStyle::Primary)
            .label("Повлиять на знамение")
        )).await?;

    let result_message = formatdoc!(r#"
        __**Знамение изменено**__
        *<@{}> попытался повлиять на судьбу, {}*
        Его шаманская сила {} и равна {}


        {}"#,
        interaction.user.id.get(),
        if success {"и у него получилось"} else {"но сделал только хуже"},
        if success && shaman_power_decreased {
            "уменьшилась"
        } else if !success {
            "увеличилась"
        } else {
            "не изменилась"
        },
        user_info.shaman_power,
        render_sign(res.current_sign)
    );

    Ok(Some(CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(result_message)
    )))
}