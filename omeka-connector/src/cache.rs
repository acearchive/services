use std::{
    num::NonZeroUsize,
    sync::{Arc, LazyLock, Mutex},
    time::Instant,
};

use lru::LruCache;

use super::{
    config,
    models::{Collection, Item},
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

type ModelCache<T> = Mutex<Option<CacheEntry<Arc<T>>>>;

static ITEMS_CACHE: ModelCache<Vec<Item>> = Mutex::new(None);

static COLLECTIONS_CACHE: ModelCache<Vec<Collection>> = Mutex::new(None);

// Fetching every item or collection is slow, so these locks are held while refreshing the
// corresponding cache to keep concurrent requests from all hitting Omeka at once.
pub static ITEMS_REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
pub static COLLECTIONS_REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn put_model<T>(cache: &ModelCache<T>, value: Arc<T>) -> anyhow::Result<()> {
    let entry = CacheEntry {
        value,
        expires_at: Instant::now() + config::model_cache_ttl()?,
    };

    *cache.lock().expect("Model cache lock poisoned.") = Some(entry);

    Ok(())
}

fn get_model<T>(cache: &ModelCache<T>) -> Option<Arc<T>> {
    let mut cache = cache.lock().expect("Model cache lock poisoned.");

    match cache.as_ref() {
        Some(entry) if entry.expires_at > Instant::now() => Some(Arc::clone(&entry.value)),
        Some(_) => {
            *cache = None;
            None
        }
        None => None,
    }
}

pub fn put_items(value: Arc<Vec<Item>>) -> anyhow::Result<()> {
    put_model(&ITEMS_CACHE, value)
}

pub fn get_items() -> Option<Arc<Vec<Item>>> {
    get_model(&ITEMS_CACHE)
}

pub fn put_collections(value: Arc<Vec<Collection>>) -> anyhow::Result<()> {
    put_model(&COLLECTIONS_CACHE, value)
}

pub fn get_collections() -> Option<Arc<Vec<Collection>>> {
    get_model(&COLLECTIONS_CACHE)
}
