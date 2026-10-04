use serde::Deserialize;

use super::{cache, omeka, url};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MediaLocator {
    Long {
        slug: omeka::AceSlug,
        filename: omeka::AceFilename,
    },
    Short {
        id: omeka::AceId,
        filename: omeka::AceFilename,
    },
    Raw {
        id: omeka::AceId,
        filename: omeka::AceFilename,
    },
}

impl MediaLocator {
    pub fn filename(&self) -> &omeka::AceFilename {
        match self {
            MediaLocator::Long { filename, .. } => filename,
            MediaLocator::Short { filename, .. } => filename,
            MediaLocator::Raw { filename, .. } => filename,
        }
    }

    pub fn with_key(self, item_key: ItemKey) -> Self {
        match self {
            MediaLocator::Long { filename, .. } => MediaLocator::Long {
                slug: item_key.unwrap_slug(),
                filename,
            },
            MediaLocator::Short { filename, .. } => MediaLocator::Short {
                id: item_key.unwrap_id(),
                filename,
            },
            MediaLocator::Raw { filename, .. } => MediaLocator::Raw {
                id: item_key.unwrap_id(),
                filename,
            },
        }
    }

    pub fn with_key_and_filename(self, item_key: ItemKey, filename: omeka::AceFilename) -> Self {
        match self {
            MediaLocator::Long { .. } => MediaLocator::Long {
                slug: item_key.unwrap_slug(),
                filename,
            },
            MediaLocator::Short { .. } => MediaLocator::Short {
                id: item_key.unwrap_id(),
                filename,
            },
            MediaLocator::Raw { .. } => MediaLocator::Raw {
                id: item_key.unwrap_id(),
                filename,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ItemKey {
    ById(omeka::AceId),
    BySlug(omeka::AceSlug),
}

impl ItemKey {
    pub fn unwrap_id(self) -> omeka::AceId {
        match self {
            ItemKey::ById(id) => id,
            ItemKey::BySlug(_) => panic!("Exepcted this item key to have a slug."),
        }
    }

    pub fn unwrap_slug(self) -> omeka::AceSlug {
        match self {
            ItemKey::ById(_) => panic!("Expected this item key to have an ID."),
            ItemKey::BySlug(slug) => slug,
        }
    }
}

impl From<MediaLocator> for ItemKey {
    fn from(locator: MediaLocator) -> Self {
        match locator {
            MediaLocator::Long { slug, .. } => ItemKey::BySlug(slug),
            MediaLocator::Short { id, .. } => ItemKey::ById(id),
            MediaLocator::Raw { id, .. } => ItemKey::ById(id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CanonicalUrl {
    ShouldRedirect(reqwest::Url),
    AlreadyCanonical,
}

#[derive(Debug, Clone)]
pub struct MediaLocation {
    pub omeka_url: reqwest::Url,
    // If `None`, we're already at the canonical URL. If `Some`, we need to redirect.
    pub canonical_url: CanonicalUrl,
}

#[derive(Debug)]
pub struct Resolver {
    client: omeka::Client,
}

impl Resolver {
    pub fn new(client: omeka::Client) -> Self {
        Resolver { client }
    }

    /// Given the ID or slug of an item in the collection, return its internal Omeka ID. If given a
    /// slug that does not resolve, check it against slug aliases. If it resolves via an alias, also
    /// return the canonical slug for that item.
    async fn resolve_canonical_item(
        &self,
        key: ItemKey,
    ) -> anyhow::Result<Option<(omeka::InternalId, Option<ItemKey>)>> {
        #[derive(Debug, Deserialize)]
        struct PropertyValue<T> {
            #[serde(rename = "@value")]
            value: T,
        }

        #[derive(Debug, Deserialize)]
        struct ItemResponse {
            #[serde(rename = "o:id")]
            internal_id: omeka::InternalId,

            #[serde(rename = "ace:slug")]
            ace_slug: Vec<PropertyValue<omeka::AceSlug>>,
        }

        let (internal_item_id, canonical_item_key) = match key {
            ItemKey::ById(id) => {
                let response = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::AceId, &id),
                    )
                    .await?;

                let item_responses = response.json::<Vec<ItemResponse>>().await?;

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::AceId,
                        id,
                    );
                }

                // If there is no matching item, we get an empty array, not a 404 Not Found.
                let internal_id = match item_responses.first() {
                    Some(item_response) => item_response.internal_id,
                    None => {
                        return Ok(None);
                    }
                };

                (internal_id, None)
            }
            ItemKey::BySlug(slug) => {
                let response = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::Slug, &slug),
                    )
                    .await?;

                let item_responses = response.json::<Vec<ItemResponse>>().await?;

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::Slug,
                        slug,
                    );
                }

                match item_responses.first() {
                    Some(item_response) => (item_response.internal_id, None),
                    None => {
                        let response = self
                            .client
                            .find(
                                omeka::ResourceType::Item,
                                omeka::FindQuery::new().property(omeka::Property::SlugAlias, &slug),
                            )
                            .await?;

                        let item_responses = response.json::<Vec<ItemResponse>>().await?;

                        if item_responses.len() > 1 {
                            log::warn!(
                                "Multiple items found with `{}` property value `{}`. Using the first one.",
                                omeka::Property::SlugAlias,
                                slug,
                            );
                        }

                        let item_response = match item_responses.first() {
                            Some(item_response) => item_response,
                            None => {
                                return Ok(None);
                            }
                        };

                        let canonical_ace_slug = match item_response.ace_slug.first() {
                            Some(slug) => slug.value.clone(),
                            None => {
                                log::warn!(
                                    "Item with internal ID `{}` has no `{}` property.",
                                    item_response.internal_id,
                                    omeka::Property::Slug,
                                );

                                return Ok(None);
                            }
                        };

                        (
                            item_response.internal_id,
                            Some(ItemKey::BySlug(canonical_ace_slug)),
                        )
                    }
                }
            }
        };

        Ok(Some((internal_item_id, canonical_item_key)))
    }

    // Given the ID or slug of an item in the collection, a filename, and whether this is a long
    // URL, short URL, or raw URL, return the Omeka URL for the media file. If the item resolves via
    // a slug alias, also return a URL using the canonical slug for that item.
    pub async fn resolve_media(&self, key: MediaLocator) -> anyhow::Result<Option<MediaLocation>> {
        #[derive(Debug, Deserialize)]
        struct PropertyValue<T> {
            #[serde(rename = "@value")]
            value: T,
        }

        #[derive(Debug, Deserialize)]
        struct MediaResponse {
            #[serde(rename = "o:original_url")]
            original_url: String,

            #[serde(rename = "ace:filename")]
            ace_filename: Vec<PropertyValue<omeka::AceFilename>>,
        }

        if let Some(cached_media_location) = cache::get_media_location(&key) {
            return Ok(Some(cached_media_location));
        }

        let (internal_item_id, canonical_item_key) =
            match self.resolve_canonical_item(key.clone().into()).await? {
                Some(pair) => pair,
                None => return Ok(None),
            };

        let response = self
            .client
            .find(
                omeka::ResourceType::Media,
                omeka::FindQuery::new()
                    .property(omeka::Property::Filename, key.filename())
                    .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
            )
            .await?;

        let media_responses = response.json::<Vec<MediaResponse>>().await?;

        if media_responses.len() > 1 {
            log::warn!(
                "Multiple items found with `{}` property value `{}`. Using the first one.",
                omeka::Property::Filename,
                key.filename(),
            );
        }

        let (original_url, canonical_url) = match media_responses.first() {
            Some(media_response) => (
                reqwest::Url::parse(&media_response.original_url)?,
                match canonical_item_key {
                    Some(canonical_item_key) => CanonicalUrl::ShouldRedirect(url::format(
                        key.clone().with_key(canonical_item_key),
                    )?),
                    None => CanonicalUrl::AlreadyCanonical,
                },
            ),
            None => {
                let response = self
                    .client
                    .find(
                        omeka::ResourceType::Media,
                        omeka::FindQuery::new()
                            .property(omeka::Property::FilenameAlias, key.filename())
                            .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
                    )
                    .await?;

                let media_responses = response.json::<Vec<MediaResponse>>().await?;

                if media_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::FilenameAlias,
                        key.filename(),
                    );
                }

                let media_response = match media_responses.first() {
                    Some(media_response) => media_response,
                    None => {
                        return Ok(None);
                    }
                };

                let canonical_filename = match media_response.ace_filename.first() {
                    Some(filename) => filename.value.clone(),
                    None => {
                        log::warn!(
                            "Media with URL `{}` has no `{}` property.",
                            url::format(key)?,
                            omeka::Property::Filename,
                        );

                        return Ok(None);
                    }
                };

                (
                    reqwest::Url::parse(&media_response.original_url)?,
                    match canonical_item_key {
                        Some(canonical_item_key) => CanonicalUrl::ShouldRedirect(url::format(
                            key.clone()
                                .with_key_and_filename(canonical_item_key, canonical_filename),
                        )?),
                        None => CanonicalUrl::ShouldRedirect(url::format(
                            key.clone()
                                .with_key_and_filename(key.clone().into(), canonical_filename),
                        )?),
                    },
                )
            }
        };

        let media_location = MediaLocation {
            omeka_url: original_url,
            canonical_url,
        };

        cache::put_media_location(key, media_location.clone());

        Ok(Some(media_location))
    }
}
