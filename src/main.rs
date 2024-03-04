use discord::Handler;
use serenity::{all::ApplicationId, prelude::*};
use dotenv::dotenv;

mod commands;
mod db;
mod discord;
pub mod signs;
pub mod config;
pub mod discord_endpoint_server;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let config = config::AppConfig::from_env().unwrap();
    
    signs::load_signs(config.sign_pack_path()).unwrap();

    let dao = db::psql::init_with_config(config.pg().clone()).await.unwrap();
    let handler = Handler::new(Box::new(dao.clone()));

    let token = config.discord_token();

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .application_id(ApplicationId::new(config.application_id()))
        .await
        .expect("Error creating client");

    if let Some(cfg) = config.server() {
        discord_endpoint_server::start(Handler::new(Box::new(dao.clone())), client, cfg.clone()).await.unwrap();
        return;
    }
    
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}


