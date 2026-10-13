use std::{
    num::NonZeroUsize,
    sync::{LazyLock, Mutex},
    time::Instant,
};

use lru::LruCache;

use super::{
    config,
    resolver::{MediaLocation, MediaLocator},
};

struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

static MEDIA_LOCATION_CACHE: LazyLock<Mutex<LruCache<MediaLocator, CacheEntry<MediaLocation>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

pub fn put_media_location(key: MediaLocator, value: MediaLocation) -> anyhow::Result<()> {
    let cache = LazyLock::force(&MEDIA_LOCATION_CACHE);

    let entry = CacheEntry {
        value,
        expires_at: Instant::now() + config::media_cache_ttl()?,
    };

    cache
        .lock()
        .expect("Media cache lock poisoned.")
        .put(key, entry);

    Ok(())
}

pub fn get_media_location(key: &MediaLocator) -> Option<MediaLocation> {
    let cache = LazyLock::force(&MEDIA_LOCATION_CACHE);

    let mut cache = cache.lock().expect("Media cache lock poisoned.");

    match cache.get(key) {
        Some(entry) if entry.expires_at > Instant::now() => Some(entry.value.clone()),
        Some(_) => {
            cache.pop(key);
            None
        }
        None => None,
    }
}
