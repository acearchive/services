use crate::{config, omeka};

fn slugify(value: &str) -> String {
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
    Ok(reqwest::Url::parse(&format!(
        "{}/artifacts/{}/{}",
        config::files_url()?,
        slug,
        filename
    ))?)
}

pub fn file_short(
    id: &omeka::AceId,
    filename: &omeka::AceFilename,
) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "{}/a/{}/{}",
        config::files_url()?,
        id,
        filename
    ))?)
}

pub fn file_raw(id: &omeka::AceId, filename: &omeka::AceFilename) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "{}/r/{}/{}",
        config::files_url()?,
        id,
        filename
    ))?)
}

pub fn person(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/people/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}

pub fn decade(decade: u32) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/decades/{}",
        config::base_domain()?,
        slugify(decade.to_string().as_str())
    ))?)
}

pub fn identity(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/identities/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}

pub fn collection(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/collections/{}",
        config::base_domain()?,
        slugify(name)
    ))?)
}
