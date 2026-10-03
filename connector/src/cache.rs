use std::{
    num::NonZeroUsize,
    sync::{LazyLock, Mutex},
};

use lru::LruCache;

use super::{
    omeka,
    store::{ItemKey, MediaKey},
};

static URL_CACHE: LazyLock<Mutex<LruCache<UrlKey, String>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UrlKey {
    MediaById {
        id: omeka::AceId,
        filename: omeka::AceFilename,
    },
    MediaBySlug {
        slug: omeka::AceSlug,
        filename: omeka::AceFilename,
    },
}

impl From<MediaKey> for UrlKey {
    fn from(key: MediaKey) -> Self {
        match key.item {
            ItemKey::ById(id) => Self::MediaById {
                id,
                filename: key.filename,
            },
            ItemKey::BySlug(slug) => Self::MediaBySlug {
                slug,
                filename: key.filename,
            },
        }
    }
}

pub fn put_url(key: UrlKey, url: &reqwest::Url) {
    let cache = LazyLock::force(&URL_CACHE);

    // We serialize the URL to a string to save memory, as `reqwest::Url` seems to internally store
    // both the serialized URL and the parsed components.
    cache
        .lock()
        .expect("Media cache lock poisoned.")
        .put(key, url.to_string());
}

pub fn get_url(key: UrlKey) -> Option<reqwest::Url> {
    let cache = LazyLock::force(&URL_CACHE);

    cache
        .lock()
        .expect("Media cache lock poisoned.")
        .get(&key)
        .map(|raw| reqwest::Url::parse(raw).expect("Failed to parse cached URL."))
}
