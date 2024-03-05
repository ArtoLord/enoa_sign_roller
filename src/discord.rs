use anyhow::{anyhow, Context, Result};
use log::{debug, error, info};
use serenity::{all::{CacheHttp, ComponentInteractionData, ComponentInteractionDataKind, CreateInteractionResponse, EventHandler, Http, Interaction, Ready}, async_trait};

use crate::{commands::{self, utils}, db::Dao};

pub struct Handler {
    dao: Box<dyn Dao>
}

impl Handler {
    pub fn new(dao: Box<dyn Dao>) -> Self {
        Handler { dao }
    }

    pub async fn init_guilds(&self, ctx: &(impl AsRef<Http> + CacheHttp)) -> Result<()> {
        let guilds = ctx.http().get_guilds(None, None).await
            .with_context(|| "Cannot get bot guilds list")?;

        for guild in guilds {
            info!("Initing commands for guild {}", guild.id);

            guild.id.set_commands(ctx, vec![
                commands::sign_roll::register(),
                commands::sign_current::register()
            ])
                .await.with_context(|| format!("Cannot init commands in guild {}", guild.id))?;
        }

        info!("Commands for guilds inited");
        Ok(())
    }

    pub async fn handle_interaction(&self, ctx: impl CacheHttp, mut interaction: Interaction) -> CreateInteractionResponse {
        debug!("Created interraction {:#?}", interaction);

        let res = match &mut interaction {
            Interaction::Command(command) => {
                info!("Received command interaction: {}", command.data.name);
                match command.data.name.as_str() {
                    "sign_roll" => commands::sign_roll::run(&self, &ctx, &command).await,
                    "sign_current" => commands::sign_current::run(&self, &ctx, &command).await,
                    cmd => Err(anyhow!(format!("Command {} not found", cmd))),
                }
            },
            Interaction::Component(component) => {
                match &component.data {
                    ComponentInteractionData {custom_id, kind: ComponentInteractionDataKind::Button, ..} => {
                        match custom_id.as_str() {
                            "change_sign" => commands::modify_sign::run(&self, &ctx, component).await,
                            cmd => Err(anyhow!(format!("Component not found {}", cmd)))
                        }
                    }
                    s => Err(anyhow!(format!("Component not found {:?}", s.kind)))
                }
            },
            Interaction::Ping(_) => Ok(CreateInteractionResponse::Pong),
            i => Err(anyhow!(format!("Interraction {:?} not supported", i)))
        };

        if res.is_err() {
            let err = res.unwrap_err();
            error!("Cannot process interraction {:?}: {}", &interaction, err);
            return utils::format_error("Что-то пошло не так");
        }

        res.unwrap()
    }

    pub fn dao(&self) -> &Box<dyn Dao> {
        &self.dao
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: serenity::all::Context, interaction: Interaction) {
        let res = self.handle_interaction(&ctx, interaction.clone()).await;
        let _ = send_resp(interaction, res, &ctx).await;
    }

    async fn ready(&self, ctx: serenity::all::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        self.init_guilds(&ctx.clone()).await.expect("Cannot init commands for guilds");
    }
}

async fn send_resp(interaction: Interaction, resp: CreateInteractionResponse, ctx: &impl CacheHttp) -> Result<()> {
    match interaction {
        Interaction::Command(cmd) => cmd.create_response(ctx, resp).await?,
        Interaction::Component(component) => component.create_response(ctx, resp).await?,
        _ => Err(anyhow!("Cannot send response to unknown interaction"))?,
    };

    Ok(())
}