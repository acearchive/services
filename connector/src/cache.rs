use std::{
    num::NonZeroUsize,
    sync::{LazyLock, Mutex},
};

use lru::LruCache;

use super::resolver::{MediaLocation, MediaLocator};

static MEDIA_LOCATION_CACHE: LazyLock<Mutex<LruCache<MediaLocator, MediaLocation>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

pub fn put_media_location(key: MediaLocator, value: MediaLocation) {
    let cache = LazyLock::force(&MEDIA_LOCATION_CACHE);

    cache
        .lock()
        .expect("Media cache lock poisoned.")
        .put(key, value);
}

pub fn get_media_location(key: &MediaLocator) -> Option<MediaLocation> {
    let cache = LazyLock::force(&MEDIA_LOCATION_CACHE);

    cache
        .lock()
        .expect("Media cache lock poisoned.")
        .get(key)
        .cloned()
}
