use std::env;
use discord::Handler;
use serenity::prelude::*;
use dotenv::dotenv;

mod commands;
mod db;
mod discord;
pub mod signs;
pub mod config;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let config = config::AppConfig::from_env().unwrap();
    
    signs::load_signs(config.sign_pack_path()).unwrap();

    let dao = db::psql::init_with_config(config.pg().clone()).await.unwrap();
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


