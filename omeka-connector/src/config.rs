use std::{env, path::PathBuf, sync::OnceLock};

struct Config {
    base_domain: String,
    omeka_url: reqwest::Url,
    files_url: reqwest::Url,
    port: u16,
    scratch_dir: PathBuf,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config {
        base_domain: env::var("BASE_DOMAIN")?,
        omeka_url: reqwest::Url::parse(&env::var("OMEKA_URL")?)?,
        files_url: reqwest::Url::parse(&env::var("FILES_URL")?)?,
        port: env::var("PORT")?.parse::<u16>()?,
        scratch_dir: env::var("SCRATCH_DIR")?.into(),
    };

    CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Config already initialized."))?;

    Ok(())
}

pub fn base_domain() -> anyhow::Result<String> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .base_domain
        .clone())
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

pub fn scratch_dir() -> anyhow::Result<PathBuf> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .scratch_dir
        .clone())
}
