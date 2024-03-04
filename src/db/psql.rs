use chrono::{DateTime, Local};
use deadpool_postgres::Pool;
use anyhow::{anyhow, Result};
use serenity::async_trait;
use tokio_postgres::NoTls;
use crate::db::Dao;
use anyhow::Context;
use std::{ops::DerefMut, time::SystemTime};

use super::{GuildInfo, SignInfo, SignState, UserInfo};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

#[derive(Clone)]
pub struct PsqlDao {
    pool: Pool
}

#[async_trait]
impl Dao for PsqlDao {
    async fn save_user_info(&self, user_info: UserInfo) -> Result<()> {
        let client = self.pool.get().await
            .with_context(|| "Cannot get connection")?;

        let stmt = client.prepare(r#"
            INSERT INTO users (id, guild_id, shaman_power)
            VALUES ($1, $2, $3)
            ON CONFLICT (id, guild_id) DO UPDATE
            SET shaman_power = $3
        "#).await?;

        client.execute(&stmt, &[&user_info.id.to_string(), &user_info.guild_id.to_string(), &user_info.shaman_power]).await?;

        Ok(())
    }

    async fn get_user_info(&self, user_id: u64, guild_id: u64) -> Result<Option<UserInfo>> {
        let client = self.pool.get().await
            .with_context(|| "Cannot get connection")?;

        let stmt = client.prepare(r#"
            SELECT id, guild_id, shaman_power 
            FROM users
            WHERE id = $1 AND guild_id = $2
        "#).await?;

        let res = client.query_opt(&stmt, &[&user_id.to_string(), &guild_id.to_string()]).await?;

        if res.is_none() {
            return Ok(None);
        }

        let row = res.unwrap();

        let shaman_power: i32 = row.get(2);

        Ok(Some(UserInfo {
            id: user_id,
            guild_id,
            shaman_power: shaman_power
        }))
    }

    /**
     * Create sign with given data
     * Returns new GuildInfo or None on conflict (if sign already created today)
     */
    async fn create_sign(&self, guild_id: u64, sign_id: String, sign_created_by: u64) -> Result<Option<GuildInfo>> {
        let client = self.pool.get().await
            .with_context(|| "Cannot get connection")?;

        let stmt = client.prepare(r#"
            INSERT INTO guilds (id, sign_id, sign_created_by_id, sign_created_at, sign_state)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT(id) DO UPDATE
            SET sign_id = $2, sign_created_by_id = $3, sign_created_at = NOW(), sign_state = $4
            WHERE guilds.sign_created_at < NOW()::date
            RETURNING (guilds.sign_created_at)
        "#).await?;

        let res = client.query_opt(&stmt, &[
                &guild_id.to_string(),
                &sign_id,
                &sign_created_by.to_string(),
                &"Created"
            ]).await?;

        // This query returns smth only if row inserted or updated
        // So we don't need to select GuildInfo twise
        if res.is_none() {
            return Ok(None);
        }

        let row = res.unwrap();

        let created_at: SystemTime = row.get(0);

        Ok(Some(GuildInfo {
            guild_id,
            current_sign: SignInfo {
                id: sign_id,
                created_by_user_id: sign_created_by,
                state: SignState::Created,
                created_at: created_at
            },
        }))
    }

    async fn get_guild_info(&self, guild_id: u64) -> Result<Option<GuildInfo>> {
        let client = self.pool.get().await
            .with_context(|| "Cannot get connection")?;

        let stmt = client.prepare(r#"
            SELECT sign_id, sign_created_at, sign_created_by_id, sign_state, sign_state_made_by_id
            FROM guilds
            WHERE id = $1
        "#).await?;

        let res = client.query_opt(&stmt, &[&guild_id.to_string()]).await?;

        if res.is_none() {
            return Ok(None);
        }

        let row = res.unwrap();

        let sign_id: String = row.get(0);
        let sign_created_at: SystemTime = row.get(1);
        let sign_created_by_id: String = row.get(2);
        let sign_state: String = row.get(3);
        let sign_state_made_by_id: Option<String> = row.get(4);

        let dt_local: DateTime<Local> = sign_created_at.clone().into();

        if dt_local.date_naive() < chrono::offset::Local::now().date_naive() {
            return Ok(None);
        }

        Ok(Some(GuildInfo {
            guild_id,
            current_sign: SignInfo {
                id: sign_id,
                created_by_user_id: sign_created_by_id.parse()?,
                state: match sign_state {
                    s if s == "Created" => SignState::Created,
                    s if s == "Success" => SignState::Success { by_user_id: sign_state_made_by_id.ok_or(
                        anyhow!("State changer not set")
                    )?.parse()? },
                    s if s == "Failed" => SignState::Failed { by_user_id: sign_state_made_by_id.ok_or(
                        anyhow!("State changer not set")
                    )?.parse()? },
                    _ => unreachable!()
                },
                created_at: sign_created_at
            },
        }))
    }

    /**
     * Change sign state
     * New state must not be Created
     * Returns new GuildInfo or Err with old GuildInfo on conflict
     */
    async fn change_sign_state(&self, guild_id: u64, new_state: SignState) -> Result<Result<GuildInfo, Option<GuildInfo>>> {
        let client = self.pool.get().await
            .with_context(|| "Cannot get connection")?;

        let stmt = client.prepare(r#"
            UPDATE guilds
            SET sign_state = $1, sign_state_made_by_id = $2
            WHERE id = $3 AND sign_state = 'Created' AND sign_created_at >= NOW()::date
            RETURNING sign_id, sign_created_at, sign_created_by_id
        "#).await?;

        let (state, state_made_by) = match new_state {
            SignState::Created => Err(anyhow!("New state canot be Created")),
            SignState::Success { by_user_id } => Ok(("Success", by_user_id.to_string())),
            SignState::Failed { by_user_id } => Ok(("Success", by_user_id.to_string())),
        }?;

        let res = client.query_opt(&stmt, &[
                &state,
                &state_made_by,
                &guild_id.to_string(),
            ]).await?;

        if res.is_none() {
            // In this case sign was not updated
            // We will just select current state
            // It can be inconsistent, but i am too lazy to make proper sql request :)
            return Ok(Err(self.get_guild_info(guild_id).await?));
        }

        let row = res.unwrap();

        let sign_id: String = row.get(0);
        let sign_created_at: SystemTime = row.get(1);
        let sign_created_by_id: String = row.get(2);

        Ok(Ok(GuildInfo {
            guild_id,
            current_sign: SignInfo {
                id: sign_id,
                created_by_user_id: sign_created_by_id.parse()?,
                state: new_state,
                created_at: sign_created_at
            },
        }))
    }
}

pub async fn init_with_config(cfg: deadpool_postgres::Config) -> Result<PsqlDao> {
    let pool = cfg.create_pool(NoTls)
    .with_context(|| "Cannot create psql connection pool")?;

    let mut conn = pool.get().await
        .with_context(|| "Cannot connect to psql")?;
    let client = conn.deref_mut().deref_mut();
    embedded::migrations::runner().run_async(client).await
    .with_context(|| "Cannot migrate database")?;

    Ok(PsqlDao {pool})
}