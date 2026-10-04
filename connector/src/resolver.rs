use serde::Deserialize;

use super::{cache, omeka};

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
}

#[derive(Debug, Clone)]
pub enum ItemKey {
    ById(omeka::AceId),
    BySlug(omeka::AceSlug),
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
pub struct MediaLocation {
    pub omeka_url: reqwest::Url,
    pub canonical_url: reqwest::Url,
}

#[derive(Debug)]
pub struct Resolver {
    client: omeka::Client,
}

impl Resolver {
    pub fn new(client: omeka::Client) -> Self {
        Resolver { client }
    }

    async fn resolve_canonical_item(
        &self,
        key: ItemKey,
    ) -> anyhow::Result<Option<(ItemKey, omeka::InternalId)>> {
        #[derive(Debug, Deserialize)]
        struct PropertyValue<T> {
            #[serde(rename = "@value")]
            value: T,
        }

        #[derive(Debug, Deserialize)]
        struct ItemResponse {
            #[serde(rename = "o:id")]
            internal_id: omeka::InternalId,

            #[serde(rename = "ace:id")]
            ace_id: Vec<PropertyValue<omeka::AceId>>,

            #[serde(rename = "ace:slug")]
            ace_slug: Vec<PropertyValue<omeka::AceSlug>>,
        }

        let (canonical_item_key, internal_item_id) = match key {
            ItemKey::ById(id) => {
                let response = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::AceId, id.to_string()),
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

                (ItemKey::ById(id), internal_id)
            }
            ItemKey::BySlug(slug) => {
                let response = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::AceSlug, &slug),
                    )
                    .await?;

                let item_responses = response.json::<Vec<ItemResponse>>().await?;

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::AceSlug,
                        &slug,
                    );
                }

                match item_responses.first() {
                    Some(item_response) => (ItemKey::BySlug(slug), item_response.internal_id),
                    None => {
                        let response = self
                            .client
                            .find(
                                omeka::ResourceType::Item,
                                omeka::FindQuery::new()
                                    .property(omeka::Property::AceSlugAlias, &slug),
                            )
                            .await?;

                        let item_responses = response.json::<Vec<ItemResponse>>().await?;

                        if item_responses.len() > 1 {
                            log::warn!(
                                "Multiple items found with `{}` property value `{}`. Using the first one.",
                                omeka::Property::AceSlugAlias,
                                &slug,
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
                                    omeka::Property::AceSlug,
                                );

                                return Ok(None);
                            }
                        };

                        (
                            ItemKey::BySlug(canonical_ace_slug),
                            item_response.internal_id,
                        )
                    }
                }
            }
        };

        Ok(Some((canonical_item_key, internal_item_id)))
    }

    pub async fn resolve_media(&self, key: MediaLocator) -> anyhow::Result<Option<MediaLocation>> {
        if let Some(cached_omeka_url) = cache::get_url(key.clone().into()) {
            return Ok(Some(MediaLocation {
                omeka_url: cached_omeka_url.clone(),
                // We only cache canonical URLs.
                canonical_url: cached_omeka_url,
            }));
        }

        let (canonical_item_key, internal_item_id) =
            match self.resolve_canonical_item(key.clone().into()).await? {
                Some(pair) => pair,
                None => return Ok(None),
            };

        let response = self
            .client
            .find(
                omeka::ResourceType::Item,
                omeka::FindQuery::new()
                    .property(omeka::Property::AceFilename, key.filename())
                    .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
            )
            .await?;

        todo!()
    }
}
