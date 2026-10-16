use std::{env, path::PathBuf, sync::OnceLock, time::Duration};

struct Config {
    base_domain: String,
    omeka_url: reqwest::Url,
    files_url: reqwest::Url,
    port: u16,
    scratch_dir: PathBuf,
    media_cache_ttl: Duration,
    model_cache_ttl: Duration,
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
        media_cache_ttl: Duration::from_secs(env::var("MEDIA_CACHE_TTL_SECS")?.parse::<u64>()?),
        model_cache_ttl: Duration::from_secs(env::var("MODEL_CACHE_TTL_SECS")?.parse::<u64>()?),
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

pub fn media_cache_ttl() -> anyhow::Result<Duration> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .media_cache_ttl)
}

pub fn model_cache_ttl() -> anyhow::Result<Duration> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .model_cache_ttl)
}
