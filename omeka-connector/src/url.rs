use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::{config, omeka};

/// The characters Go's `url.PathEscape` escapes, to match Hugo's `urls.PathEscape`.
const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b':')
    .remove(b'=')
    .remove(b'@');

fn encode_segment(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT).to_string()
}

pub fn artifact_long(slug: &omeka::AceSlug) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/artifacts/{}",
        config::base_domain(),
        slug
    ))?)
}

pub fn artifact_short(id: &omeka::AceId) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/a/{}",
        config::base_domain(),
        id
    ))?)
}

pub fn file_long(
    slug: &omeka::AceSlug,
    filename: &omeka::AceFilename,
) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url().clone();
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
    let mut url = config::files_url().clone();
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Cannot modify URL path segments."))?
        .pop_if_empty()
        .push("a")
        .push(id.to_string().as_str())
        .push(filename.to_string().as_str());

    Ok(url)
}

pub fn file_raw(id: &omeka::AceId, filename: &omeka::AceFilename) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url().clone();
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
        config::base_domain(),
        encode_segment(name)
    ))?)
}

pub fn identity(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/identities/{}",
        config::base_domain(),
        encode_segment(name)
    ))?)
}

pub fn identities() -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/identities",
        config::base_domain(),
    ))?)
}

pub fn collection(name: &str) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/collections/{}",
        config::base_domain(),
        encode_segment(name)
    ))?)
}
