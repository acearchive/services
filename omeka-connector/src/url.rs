use crate::{config, omeka};

pub fn slugify(value: &str) -> String {
    value.to_lowercase().replace(' ', "-")
}

pub fn artifact_long(slug: &omeka::AceSlug) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/artifacts/{}",
        config::base_domain()?,
        slug
    ))?)
}

pub fn artifact_short(id: &omeka::AceId) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/a/{}",
        config::base_domain()?,
        id
    ))?)
}

pub fn file_long(
    slug: &omeka::AceSlug,
    filename: &omeka::AceFilename,
) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url()?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Cannot modify URL path segments."))?
        .pop_if_empty()
        .push("artifacts")
        .push(slug.to_string().as_str())
        .push(filename.to_string().as_str());

    Ok(url)
}

pub fn file_short(
    id: &omeka::AceId,
    filename: &omeka::AceFilename,
) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url()?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Cannot modify URL path segments."))?
        .pop_if_empty()
        .push("a")
        .push(id.to_string().as_str())
        .push(filename.to_string().as_str());

    Ok(url)
}

pub fn file_raw(id: &omeka::AceId, filename: &omeka::AceFilename) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url()?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Cannot modify URL path segments."))?
        .pop_if_empty()
        .push("r")
        .push(id.to_string().as_str())
        .push(filename.to_string().as_str());

    Ok(url)
}

pub fn person(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/people/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}

pub fn identity(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/identities/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}

pub fn identities() -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/identities",
        config::base_domain()?,
    ))?)
}

pub fn collection(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/collections/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}
