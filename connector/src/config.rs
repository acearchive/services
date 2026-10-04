use std::{collections::HashMap, sync::OnceLock};

use crate::omeka;

static CONFIG: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn init_config() -> anyhow::Result<()> {
    let env = dotenvy::dotenv_iter()?
        .filter_map(|item| match item {
            Ok((key, value)) => Some((key, value)),
            Err(err) => {
                log::error!("Failed to parse dotenv file: {}", err);
                None
            }
        })
        .collect::<HashMap<String, String>>();

    CONFIG
        .set(env)
        .map_err(|_| anyhow::anyhow!("Config already initialized."))?;

    Ok(())
}

fn get_config(key: &str) -> anyhow::Result<String> {
    CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Not set in config: {}", key))
}

pub fn omeka_url() -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&get_config("OMEKA_URL")?)?)
}

pub fn files_url() -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&get_config("FILES_URL")?)?)
}

pub fn port() -> anyhow::Result<u16> {
    Ok(get_config("PORT")?.parse::<u16>()?)
}

pub fn omeka_key_id() -> anyhow::Result<omeka::ApiKeyId> {
    Ok(get_config("OMEKA_KEY_ID")?.into())
}

pub fn omeka_key_cred() -> anyhow::Result<omeka::ApiKeyCred> {
    Ok(get_config("OMEKA_KEY_CRED")?.into())
}
