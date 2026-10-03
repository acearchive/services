use serde::Deserialize;

use super::omeka;

#[derive(Debug, Clone)]
pub enum ItemKey {
    ById(omeka::AceId),
    BySlug(omeka::AceSlug),
}

#[derive(Debug, Clone)]
pub struct MediaKey {
    pub item: ItemKey,
    pub filename: omeka::AceFilename,
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

                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    return Ok(None);
                }

                let item_responses = response.json::<Vec<ItemResponse>>().await?;

                let internal_id = match item_responses.first() {
                    Some(item_response) => item_response.internal_id,
                    None => {
                        return Ok(None);
                    }
                };

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::AceId,
                        id,
                    );
                }

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

                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    let response = self
                        .client
                        .find(
                            omeka::ResourceType::Item,
                            omeka::FindQuery::new().property(omeka::Property::AceSlugAlias, &slug),
                        )
                        .await?;

                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Ok(None);
                    }

                    let item_responses = response.json::<Vec<ItemResponse>>().await?;

                    let item_response = match item_responses.first() {
                        Some(item_response) => item_response,
                        None => {
                            return Ok(None);
                        }
                    };

                    if item_responses.len() > 1 {
                        log::warn!(
                            "Multiple items found with `{}` property value `{}`. Using the first one.",
                            omeka::Property::AceSlugAlias,
                            &slug,
                        );
                    }

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
                } else {
                    (
                        ItemKey::BySlug(slug),
                        response.json::<ItemResponse>().await?.internal_id,
                    )
                }
            }
        };

        Ok(Some((canonical_item_key, internal_item_id)))
    }

    pub async fn resolve_media(&self, key: MediaKey) -> anyhow::Result<Option<MediaLocation>> {
        let (canonical_item_key, internal_item_id) =
            match self.resolve_canonical_item(key.item).await? {
                Some((canonical_item_key, internal_id)) => (canonical_item_key, internal_id),
                None => return Ok(None),
            };

        todo!()
    }
}
