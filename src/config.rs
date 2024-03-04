use serde::Deserialize;
use anyhow::Result;


#[derive(Debug, Deserialize)]
pub struct AppConfig {
    sign_pack_path: String,
    pg: deadpool_postgres::Config,
    discord_token: String,
    application_id: u64,
    server: Option<ServerConf>
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConf {
    address: String,
    discord_pk: String,
}

impl AppConfig {
    pub fn from_env() -> Result<AppConfig> {
        Ok(config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()?)
    }

    pub fn sign_pack_path(&self) -> String {
        self.sign_pack_path.clone()
    }

    pub fn pg(&self) -> &deadpool_postgres::Config {
        &self.pg
    }

    pub fn discord_token(&self) -> String {
        self.discord_token.clone()
    }

    pub fn application_id(&self) -> u64 {
        self.application_id
    }

    pub fn server(&self) -> &Option<ServerConf> {
        &self.server
    }
}

impl ServerConf {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn discord_pk(&self) -> String {
        self.discord_pk.clone()
    }
}