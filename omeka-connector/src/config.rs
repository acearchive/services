use std::{env, path::PathBuf, sync::OnceLock, time::Duration};

#[derive(Debug, Clone)]
pub struct DataExportConfig {
    pub cron_spec: croner::Cron,
    pub bucket: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub healthcheck_ping_url: Option<reqwest::Url>,
}

struct Config {
    base_domain: String,
    omeka_url: reqwest::Url,
    files_url: reqwest::Url,
    port: u16,
    scratch_dir: PathBuf,
    media_cache_ttl: Duration,
    model_cache_ttl: Duration,
    data_export: Option<DataExportConfig>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn optional_var(key: &str) -> anyhow::Result<Option<String>> {
    match env::var(key) {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

// Unset and empty variables are treated the same, since Compose passes through empty strings.
fn required_var(key: &str) -> anyhow::Result<String> {
    optional_var(key)?.ok_or_else(|| anyhow::anyhow!("{} must be set.", key))
}

fn init_data_export_config() -> anyhow::Result<Option<DataExportConfig>> {
    let Some(cron_spec) = optional_var("DATA_EXPORT_CRON_SPEC")? else {
        return Ok(None);
    };

    Ok(Some(DataExportConfig {
        cron_spec: cron_spec
            .parse()
            .map_err(|err| anyhow::anyhow!("Invalid DATA_EXPORT_CRON_SPEC: {}", err))?,
        bucket: required_var("DATA_EXPORT_BUCKET")?,
        s3_endpoint: required_var("DATA_EXPORT_S3_ENDPOINT")?,
        s3_region: optional_var("DATA_EXPORT_S3_REGION")?.unwrap_or_else(|| String::from("auto")),
        s3_access_key_id: required_var("DATA_EXPORT_S3_ACCESS_KEY_ID")?,
        s3_secret_access_key: required_var("DATA_EXPORT_S3_SECRET_ACCESS_KEY")?,
        healthcheck_ping_url: optional_var("DATA_EXPORT_HEALTHCHECK_PING_URL")?
            .map(|url| reqwest::Url::parse(&url))
            .transpose()?,
    }))
}

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
        data_export: init_data_export_config()?,
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

/// Returns `None` if automatic data exports are disabled.
pub fn data_export() -> anyhow::Result<Option<DataExportConfig>> {
    Ok(CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized."))?
        .data_export
        .clone())
}
