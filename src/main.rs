use std::env;

use db::Dao;
use discord::Handler;
use serenity::all::{EventHandler, GuildId};
use serenity::async_trait;
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use dotenv::dotenv;

mod commands;
mod db;
mod discord;
pub mod signs;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let dao = db::psql::init().await.unwrap();
    let handler = Handler::new(Box::new(dao));

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}


