use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage};


pub fn format_error(msg: impl Into<String>) -> CreateInteractionResponse {
    let mut new_msg = "**Ошибка:**\n".to_owned();
    new_msg.push_str(&Into::<String>::into(msg));

    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(new_msg)
            .ephemeral(true)
    )
}