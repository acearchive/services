use std::{
    fs,
    io::{self, Write},
    path::{Component, Path},
    time::Duration,
};

use chrono::{DateTime, Datelike, Days, NaiveDateTime, NaiveTime, Utc};
use s3::{Bucket, Region, creds::Credentials};
use tokio::runtime::Handle;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::{
    assets,
    config::{self, DataExportConfig},
    models::Item,
    omeka,
    rdf::{ModelOptions, ModelSerializer},
    resolver::Resolver,
};

const STAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
const LATEST_KEY: &str = "latest.zip";
const SCRATCH_PREFIX: &str = "data-export-";
const PING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tier {
    Daily,
    Weekly,
    Monthly,
}

impl Tier {
    fn as_str(&self) -> &'static str {
        match self {
            Tier::Daily => "daily",
            Tier::Weekly => "weekly",
            Tier::Monthly => "monthly",
        }
    }

    fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "daily" => Some(Tier::Daily),
            "weekly" => Some(Tier::Weekly),
            "monthly" => Some(Tier::Monthly),
            _ => None,
        }
    }
}

/// Run data exports on the configured schedule, forever.
pub async fn run_scheduler(export_config: DataExportConfig) {
    if let Err(err) = remove_stale_archives() {
        log::warn!("Failed to clean up stale data exports: {:#}", err);
    }

    loop {
        let now = Utc::now();

        let next = match export_config.cron_spec.find_next_occurrence(&now, false) {
            Ok(next) => next,
            Err(err) => {
                log::error!("No upcoming data export matches the cron spec: {}", err);
                return;
            }
        };

        log::info!("Next data export scheduled for {}.", next.to_rfc3339());

        tokio::time::sleep((next - now).to_std().unwrap_or_default()).await;

        ping(&export_config, PingKind::Start).await;

        match export(&export_config).await {
            Ok(key) => {
                log::info!("Data export uploaded as `{}`.", key);
                ping(&export_config, PingKind::Success).await;
            }
            Err(err) => {
                log::error!("Data export failed: {:#}", err);
                ping(&export_config, PingKind::Failure).await;
            }
        }
    }
}

/// Build an archive and upload it, returning the key it was uploaded under.
async fn export(export_config: &DataExportConfig) -> anyhow::Result<String> {
    let now = Utc::now();
    let stamp = now.format(STAMP_FORMAT).to_string();
    let bucket = new_bucket(export_config)?;

    // Check that we can reach the bucket before doing the expensive work of building an archive.
    let tier = choose_tier(&bucket, now).await?;
    let key = format!("{}/{}.zip", tier.as_str(), stamp);

    let scratch_dir = config::scratch_dir();
    fs::create_dir_all(scratch_dir)?;
    let path = scratch_dir.join(format!("{}{}.zip", SCRATCH_PREFIX, stamp));

    let result = async {
        log::info!("Building data export at `{}`.", path.display());
        build_archive(&path, now).await?;

        log::info!("Uploading data export as `{}`.", key);
        upload(&bucket, &path, &key).await?;

        log::info!("Uploading data export as `{}`.", LATEST_KEY);
        upload(&bucket, &path, LATEST_KEY).await?;

        anyhow::Ok(())
    }
    .await;

    if let Err(err) = tokio::fs::remove_file(&path).await
        && err.kind() != io::ErrorKind::NotFound
    {
        log::warn!(
            "Failed to remove local data export `{}`: {}",
            path.display(),
            err
        );
    }

    result.map(|()| key)
}

fn new_bucket(export_config: &DataExportConfig) -> anyhow::Result<Box<Bucket>> {
    let region = Region::Custom {
        region: export_config.s3_region.clone(),
        endpoint: export_config.s3_endpoint.clone(),
    };

    let credentials = Credentials::new(
        Some(&export_config.s3_access_key_id),
        Some(&export_config.s3_secret_access_key),
        None,
        None,
        None,
    )?;

    Ok(Bucket::new(&export_config.bucket, region, credentials)?.with_path_style())
}

/// Parse an object key of the form `{tier}/{stamp}.zip`.
fn parse_key(key: &str) -> Option<(Tier, NaiveDateTime)> {
    let (prefix, name) = key.split_once('/')?;
    let stamp = name.strip_suffix(".zip")?;

    Some((
        Tier::from_prefix(prefix)?,
        NaiveDateTime::parse_from_str(stamp, STAMP_FORMAT).ok()?,
    ))
}

/// Choose which tier to upload a new export to.
///
/// This is a monthly export if there hasn't been one yet this month, a weekly export if there
/// hasn't been a monthly or weekly export yet this week, and a daily export otherwise. Weeks start
/// on Monday.
async fn choose_tier(bucket: &Bucket, now: DateTime<Utc>) -> anyhow::Result<Tier> {
    let today = now.date_naive();

    let month_start = today
        .with_day(1)
        .expect("Every month has a first day.")
        .and_time(NaiveTime::MIN);

    let week_start =
        (today - Days::new(today.weekday().num_days_from_monday().into())).and_time(NaiveTime::MIN);

    let mut existing = Vec::new();

    for tier in [Tier::Monthly, Tier::Weekly] {
        for page in bucket.list(format!("{}/", tier.as_str()), None).await? {
            existing.extend(page.contents.iter().filter_map(|o| parse_key(&o.key)));
        }
    }

    let has_since = |tiers: &[Tier], since: NaiveDateTime| {
        existing
            .iter()
            .any(|(tier, stamp)| tiers.contains(tier) && *stamp >= since)
    };

    Ok(if !has_since(&[Tier::Monthly], month_start) {
        Tier::Monthly
    } else if !has_since(&[Tier::Monthly, Tier::Weekly], week_start) {
        Tier::Weekly
    } else {
        Tier::Daily
    })
}

async fn build_archive(path: &Path, now: DateTime<Utc>) -> anyhow::Result<()> {
    let resolver = Resolver::new(omeka::Client::from_config(reqwest::Client::new())?);
    let items = resolver.list_all_items().await?;

    // Include project-specific terms so users can locate files in the archive.
    let mut serializer = ModelSerializer::new(ModelOptions {
        include_private_terms: true,
    });

    serializer.add_items(&items)?;

    let metadata = serializer.into_json()?;
    let path = path.to_owned();
    let handle = Handle::current();

    // The `zip` crate is synchronous, so we write the archive on a blocking thread and drive the
    // HTTP requests for each file from there.
    tokio::task::spawn_blocking(move || write_archive(&handle, &path, &items, &metadata, now))
        .await?
}

/// The path of an item's metadata file within the archive.
fn metadata_entry_path(slug: &omeka::AceSlug) -> anyhow::Result<String> {
    safe_entry_path(&["items", slug.as_ref(), "metadata.md"])
}

/// The path of a file within the archive.
fn file_entry_path(slug: &omeka::AceSlug, filename: &omeka::AceFilename) -> anyhow::Result<String> {
    safe_entry_path(&["items", slug.as_ref(), "files", filename.as_ref()])
}

/// Join path segments into a path within the archive.
fn safe_entry_path(segments: &[&str]) -> anyhow::Result<String> {
    let path = segments.join("/");

    // Defense in depth against a slug or filename that would escape `items/` when extracted.
    let is_safe = segments.iter().all(|segment| !segment.is_empty())
        && Path::new(&path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)));

    if !is_safe {
        anyhow::bail!("Refusing to add unsafe path `{}` to data export.", path);
    }

    Ok(path)
}

fn write_archive(
    handle: &Handle,
    path: &Path,
    items: &[Item],
    metadata: &serde_json::Value,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut zip = ZipWriter::new(io::BufWriter::new(fs::File::create(path)?));

    let options = SimpleFileOptions::default().last_modified_time(now.naive_utc().try_into()?);

    // Media files are generally already compressed, so we store them as-is. They may also be larger
    // than 4 GiB, which requires ZIP64.
    let options = options
        .compression_method(CompressionMethod::Stored)
        .large_file(true);

    zip.start_file("README.txt", options)?;
    zip.write_all(assets::render_export_readme(now).as_bytes())?;

    zip.start_file("metadata.jsonld", options)?;
    serde_json::to_writer_pretty(&mut zip, metadata)?;

    for item in items {
        let entry = metadata_entry_path(&item.slug)?;
        zip.start_file(entry.as_str(), options)?;
        zip.write_all(assets::render_item_readme(item).as_bytes())?;

        for file in &item.files {
            let entry = file_entry_path(&item.slug, &file.filename)?;

            let mut response = handle
                .block_on(client.get(file.omeka_url.clone()).send())?
                .error_for_status()?;

            zip.start_file(entry.as_str(), options)?;

            // Stream the file into the archive so we never hold a whole file in memory.
            while let Some(chunk) = handle.block_on(response.chunk())? {
                zip.write_all(&chunk)?;
            }

            log::info!("Added `{}` to data export.", entry);
        }
    }

    let mut writer = zip.finish()?;
    writer.flush()?;
    writer.get_ref().sync_all()?;

    Ok(())
}

async fn upload(bucket: &Bucket, path: &Path, key: &str) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::open(path).await?;

    let response = bucket
        .put_object_stream_with_content_type(&mut file, key, "application/zip")
        .await?;

    if !(200..300).contains(&response.status_code()) {
        anyhow::bail!(
            "Uploading `{}` failed with status {}.",
            key,
            response.status_code()
        );
    }

    Ok(())
}

/// Remove archives left in the scratch directory by an export that was interrupted.
fn remove_stale_archives() -> anyhow::Result<()> {
    let scratch_dir = config::scratch_dir();
    fs::create_dir_all(scratch_dir)?;

    for entry in fs::read_dir(scratch_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name.starts_with(SCRATCH_PREFIX) && name.ends_with(".zip") {
            log::info!("Removing stale data export `{}`.", name);
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum PingKind {
    Start,
    Success,
    Failure,
}

/// Ping the healthcheck endpoint, if one is configured.
async fn ping(export_config: &DataExportConfig, kind: PingKind) {
    let suffix = match kind {
        PingKind::Start => Some("start"),
        PingKind::Success => None,
        PingKind::Failure => Some("fail"),
    };

    let Some(base_url) = &export_config.healthcheck_ping_url else {
        return;
    };

    let url = match suffix {
        Some(suffix) => format!("{}/{}", base_url.as_str().trim_end_matches('/'), suffix),
        None => base_url.to_string(),
    };

    let result = reqwest::Client::new()
        .get(url)
        .timeout(PING_TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);

    // The ping URL is a secret, so keep it out of the logs.
    if let Err(err) = result {
        log::warn!("Failed to ping healthcheck endpoint: {}", err.without_url());
    }
}
