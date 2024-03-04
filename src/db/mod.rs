use std::time::SystemTime;

use serenity::async_trait;
use anyhow::Result;

pub mod psql;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: u64,
    pub guild_id: u64,
    pub shaman_power: i32
}


#[derive(Eq, PartialEq, Debug)]
pub enum SignState {
    Created,
    Success{by_user_id: u64},
    Failed{by_user_id: u64}
}

#[derive(Eq, PartialEq, Debug)]
pub struct SignInfo {
    pub id: String,
    pub created_by_user_id: u64,
    pub state: SignState,
    pub created_at: SystemTime,
}

pub struct GuildInfo {
    pub guild_id: u64,
    pub current_sign: SignInfo
}

#[async_trait]
pub trait Dao: Sync + Send {
    async fn save_user_info(&self, user_info: UserInfo) -> Result<()>;
    async fn get_user_info(&self, user_id: u64, guild_id: u64) -> Result<Option<UserInfo>>;

    /**
     * Create sign with given data
     * Returns new GuildInfo or None on conflict (if sign already created today)
     */
    async fn create_sign(&self, guild_id: u64, sign_id: String, sign_created_by: u64) -> Result<Option<GuildInfo>>;
    async fn get_guild_info(&self, guild_id: u64) -> Result<Option<GuildInfo>>;

    /**
     * Change sign state
     * Returns new GuildInfo or Err with old GuildInfo on conflict
     */
    async fn change_sign_state(&self, guild_id: u64, new_state: SignState) -> Result<Result<GuildInfo, Option<GuildInfo>>>;
}