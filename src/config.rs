use serde::Deserialize;
use anyhow::Result;


#[derive(Debug, Deserialize)]
pub struct AppConfig {
    sign_pack_path: String,
    pg: deadpool_postgres::Config
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
}