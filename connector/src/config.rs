use std::{env, path::PathBuf, sync::OnceLock};

use crate::omeka;

struct Config {
    omeka_url: reqwest::Url,
    files_url: reqwest::Url,
    port: u16,
    data_dir: PathBuf,
    omeka_key_id: omeka::ApiKeyId,
    omeka_key_cred: omeka::ApiKeyCred,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config {
        omeka_url: reqwest::Url::parse(&env::var("OMEKA_URL")?)?,
        files_url: reqwest::Url::parse(&env::var("FILES_URL")?)?,
        port: env::var("PORT")?.parse::<u16>()?,
        data_dir: env::var("DATA_DIR")?.into(),
        omeka_key_id: env::var("OMEKA_KEY_ID")?.into(),
        omeka_key_cred: env::var("OMEKA_KEY_CRED")?.into(),
    };

    CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Config already initialized."))?;

    Ok(())
}

pub fn omeka_url() -> anyhow::Result<reqwest::Url> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .omeka_url
        .clone())
}

pub fn files_url() -> anyhow::Result<reqwest::Url> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .files_url
        .clone())
}

pub fn port() -> anyhow::Result<u16> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .port)
}

pub fn data_dir() -> anyhow::Result<PathBuf> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .data_dir
        .clone())
}

pub fn omeka_key_id() -> anyhow::Result<omeka::ApiKeyId> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .omeka_key_id
        .clone())
}

pub fn omeka_key_cred() -> anyhow::Result<omeka::ApiKeyCred> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .omeka_key_cred
        .clone())
}
