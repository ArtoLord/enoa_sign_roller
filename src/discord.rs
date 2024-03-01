use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use serenity::{all::{CacheHttp, ComponentInteractionData, ComponentInteractionDataKind, Content, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Interaction, Ready}, async_trait};

use crate::{commands::{self, utils}, db::Dao};

pub struct Handler {
    dao: Box<dyn Dao>
}

impl Handler {
    pub fn new(dao: Box<dyn Dao>) -> Self {
        Handler { dao }
    }

    async fn init_guilds(&self, ctx: serenity::all::Context) -> Result<()> {
        let guilds = ctx.http().get_guilds(None, None).await
            .with_context(|| "Cannot get bot guilds list")?;

        for guild in guilds {
            info!("Initing commands for guild {}", guild.id);

            guild.id.set_commands(&ctx.http, vec![
                commands::hello_world::register(),
                commands::sign_roll::register(),
                commands::sign_current::register()
            ])
                .await.with_context(|| format!("Cannot init commands in guild {}", guild.id))?;
        }

        info!("Commands for guilds inited");
        Ok(())
    }

    pub fn dao(&self) -> &Box<dyn Dao> {
        &self.dao
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: serenity::all::Context, interaction: Interaction) {
        debug!("Created interraction {:#?}", interaction);

        let res = match &interaction {
            Interaction::Command(command) => {
                info!("Received command interaction: {}", command.data.name);
                match command.data.name.as_str() {
                    "hello" => commands::hello_world::run(&ctx, &command).await,
                    "sign_roll" => commands::sign_roll::run(&self, &ctx, &command).await,
                    "sign_current" => commands::sign_current::run(&self, &ctx, &command).await,
                    cmd => Err(anyhow!(format!("Command {} not found", cmd))),
                }
            },
            Interaction::Component(component) => {
                match &component.data {
                    ComponentInteractionData {custom_id, kind: ComponentInteractionDataKind::Button, ..} => {
                        match custom_id.as_str() {
                            "change_sign" => commands::modify_sign::run(&self, &ctx, &component).await,
                            cmd => Err(anyhow!(format!("Component not found {}", cmd)))
                        }
                    }
                    s => Err(anyhow!(format!("Component not found {:?}", s.kind)))
                }
            },
            i => Err(anyhow!(format!("Interraction {:?} not supported", i)))
        };

        if res.is_err() {
            let err = res.unwrap_err();
            error!("Cannot process interraction {:?}: {}", &interaction, err);

            let res = send_resp(interaction, utils::format_error("Что-то пошло не так"), &ctx).await;
            if res.is_err() {
                error!("Cannot send response: {:?}", res)
            }
            return;
        }

        let res = res.unwrap();
        let res = send_resp(interaction, res, &ctx).await;
        if res.is_err() {
            error!("Cannot send response: {:?}", res)
        }

    }

    async fn ready(&self, ctx: serenity::all::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        self.init_guilds(ctx.clone()).await.expect("Cannot init commands for guilds");
    }
}

async fn send_resp(interaction: Interaction, resp: CreateInteractionResponse, ctx: &serenity::all::Context) -> Result<()> {
    match interaction {
        Interaction::Command(cmd) => cmd.create_response(ctx, resp).await?,
        Interaction::Component(component) => component.create_response(ctx, resp).await?,
        _ => Err(anyhow!("Cannot send response to unknown interaction"))?,
    };

    Ok(())
}