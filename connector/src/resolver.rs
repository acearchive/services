use std::collections::HashMap;

use futures::future::{self, FutureExt};

use super::{
    cache, config,
    models::{Collection, File, Identity, Item, Link, Person},
    omeka,
};

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

    pub fn to_url(&self) -> anyhow::Result<reqwest::Url> {
        let mut url = config::files_url()?;

        {
            let mut path_segments = url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("Failed to get path segments from files URL."))?;

            match self {
                MediaLocator::Long { slug, filename } => {
                    path_segments.extend(["artifacts", slug.as_ref(), filename.as_ref()]);
                }
                MediaLocator::Short { id, filename } => {
                    path_segments.extend(["a", id.as_ref(), filename.as_ref()]);
                }
                MediaLocator::Raw { id, filename } => {
                    path_segments.extend(["r", id.as_ref(), filename.as_ref()]);
                }
            }
        }

        Ok(url)
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

fn expect_one<T>(
    property: omeka::Property,
    values: Vec<omeka::LiteralPropertyValue<T>>,
    internal_id: omeka::InternalId,
) -> Option<T> {
    if values.len() > 1 {
        log::info!(
            "Resource with internal ID `{}` has multiple values for property `{}`. Using the first one.",
            internal_id,
            property,
        );
    }

    match values.into_iter().next().map(|value| value.value) {
        Some(value) => Some(value),
        None => {
            log::info!(
                "Resource with internal ID `{}` is missing required property `{}`.",
                internal_id,
                property,
            );

            None
        }
    }
}

fn maybe_one<T>(
    property: omeka::Property,
    values: Vec<omeka::LiteralPropertyValue<T>>,
    internal_id: omeka::InternalId,
) -> Option<T>
where
    T: Clone,
{
    if values.len() > 1 {
        log::info!(
            "Resource with internal ID `{}` has multiple values for property `{}`. Using the first one.",
            internal_id,
            property,
        );
    }

    values.first().map(|value| value.value.clone())
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
        let (internal_item_id, canonical_item_key) = match key {
            ItemKey::ById(id) => {
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::AceId, &id),
                    )
                    .await?;

                let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

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
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        omeka::FindQuery::new().property(omeka::Property::Slug, &slug),
                    )
                    .await?;

                let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

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
                        let (response, _) = self
                            .client
                            .find(
                                omeka::ResourceType::Item,
                                omeka::FindQuery::new().property(omeka::Property::SlugAlias, &slug),
                            )
                            .await?;

                        let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

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

                        let canonical_ace_slug = match item_response.slug.first() {
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
        if let Some(cached_media_location) = cache::get_media_location(&key) {
            return Ok(Some(cached_media_location));
        }

        let (internal_item_id, canonical_item_key) =
            match self.resolve_canonical_item(key.clone().into()).await? {
                Some(pair) => pair,
                None => return Ok(None),
            };

        let (response, _) = self
            .client
            .find(
                omeka::ResourceType::Media,
                omeka::FindQuery::new()
                    .property(omeka::Property::Filename, key.filename())
                    .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
            )
            .await?;

        let media_responses = response.json::<Vec<omeka::MediaResponse>>().await?;

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
                    Some(canonical_item_key) => CanonicalUrl::ShouldRedirect(
                        key.clone().with_key(canonical_item_key).to_url()?,
                    ),
                    None => CanonicalUrl::AlreadyCanonical,
                },
            ),
            None => {
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Media,
                        omeka::FindQuery::new()
                            .property(omeka::Property::FilenameAlias, key.filename())
                            .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
                    )
                    .await?;

                let media_responses = response.json::<Vec<omeka::MediaResponse>>().await?;

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

                let canonical_filename = match media_response.filename.first() {
                    Some(filename) => filename.value.clone(),
                    None => {
                        log::warn!(
                            "Media with URL `{}` has no `{}` property.",
                            key.to_url()?,
                            omeka::Property::Filename,
                        );

                        return Ok(None);
                    }
                };

                (
                    reqwest::Url::parse(&media_response.original_url)?,
                    match canonical_item_key {
                        Some(canonical_item_key) => CanonicalUrl::ShouldRedirect(
                            key.clone()
                                .with_key_and_filename(canonical_item_key, canonical_filename)
                                .to_url()?,
                        ),
                        None => CanonicalUrl::ShouldRedirect(
                            key.clone()
                                .with_key_and_filename(key.clone().into(), canonical_filename)
                                .to_url()?,
                        ),
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

    pub async fn list_items(&self) -> anyhow::Result<Vec<Item>> {
        // Make all three API calls concurrently.
        let mut query = omeka::FindQuery::new();
        query.has_property(omeka::Property::AceId);
        query.has_property(omeka::Property::Slug);
        let items_future = self.client.find(omeka::ResourceType::Item, &query);

        let query = omeka::FindQuery::new();
        let item_sets_future = self.client.find(omeka::ResourceType::ItemSet, &query);

        let query = omeka::FindQuery::new();
        let media_future = self.client.find(omeka::ResourceType::Media, &query);

        let (
            (mut items_response, mut items_maybe_next_page),
            (mut item_sets_response, mut item_sets_maybe_next_page),
            (mut media_response, mut media_maybe_next_page),
        ) = tokio::try_join!(items_future, item_sets_future, media_future)?;

        let mut items_responses = Vec::new();
        let mut item_sets_responses = Vec::new();
        let mut media_responses = Vec::new();

        // Paginate the three API calls and collect their results, concurrently.
        while items_maybe_next_page.is_some()
            || item_sets_maybe_next_page.is_some()
            || media_maybe_next_page.is_some()
        {
            let items_future = if let Some(next_page) = items_maybe_next_page {
                items_responses.extend(items_response.json::<Vec<omeka::ItemResponse>>().await?);
                self.client.next_page(next_page).boxed()
            } else {
                future::ready(Ok((items_response, None))).boxed()
            };

            let item_sets_future = if let Some(next_page) = item_sets_maybe_next_page {
                item_sets_responses.extend(
                    item_sets_response
                        .json::<Vec<omeka::ItemSetResponse>>()
                        .await?,
                );
                self.client.next_page(next_page).boxed()
            } else {
                future::ready(Ok((item_sets_response, None))).boxed()
            };

            let media_future = if let Some(next_page) = media_maybe_next_page {
                media_responses.extend(media_response.json::<Vec<omeka::MediaResponse>>().await?);
                self.client.next_page(next_page).boxed()
            } else {
                future::ready(Ok((media_response, None))).boxed()
            };

            (
                (items_response, items_maybe_next_page),
                (item_sets_response, item_sets_maybe_next_page),
                (media_response, media_maybe_next_page),
            ) = tokio::try_join!(items_future, item_sets_future, media_future)?;
        }

        items_responses.extend(items_response.json::<Vec<omeka::ItemResponse>>().await?);
        item_sets_responses.extend(
            item_sets_response
                .json::<Vec<omeka::ItemSetResponse>>()
                .await?,
        );
        media_responses.extend(media_response.json::<Vec<omeka::MediaResponse>>().await?);

        let item_sets_responses_by_id: HashMap<omeka::InternalId, omeka::ItemSetResponse> =
            item_sets_responses
                .into_iter()
                .map(|item_set_response| (item_set_response.internal_id, item_set_response))
                .collect();

        let media_responses_by_id: HashMap<omeka::InternalId, omeka::MediaResponse> =
            media_responses
                .into_iter()
                .map(|media_response| (media_response.internal_id, media_response))
                .collect();

        Ok(items_responses
            .into_iter()
            .filter_map(|item_response| {
                let internal_id = item_response.internal_id;

                // Required properties.
                let id = expect_one(omeka::Property::AceId, item_response.id, internal_id)?;
                let slug = expect_one(omeka::Property::Slug, item_response.slug, internal_id)?;
                let title = expect_one(omeka::Property::Title, item_response.title, internal_id)?;
                let summary = expect_one(
                    omeka::Property::Abstract,
                    item_response.summary,
                    internal_id,
                )?;
                let (from_year, to_year) =
                    expect_one(omeka::Property::Created, item_response.created, internal_id)?
                        .split_once('/')
                        .map(|(from, to)| (from.parse::<u32>().ok(), to.parse::<u32>().ok()))
                        .unwrap_or((None, None));
                let from_year = from_year?;

                // Optional single-value properties.
                let description = maybe_one(
                    omeka::Property::Description,
                    item_response.description,
                    internal_id,
                );

                // Multi-value properties.
                let people = item_response
                    .creator
                    .into_iter()
                    .map(|value| Person {
                        id: value.id,
                        title: value.title,
                    })
                    .collect();
                let links = item_response
                    .relation
                    .into_iter()
                    .map(|value| Link {
                        title: value.title,
                        url: value.url,
                    })
                    .collect();
                let aliases = item_response
                    .slug_alias
                    .into_iter()
                    .map(|value| value.value)
                    .collect();
                let identities = item_response
                    .subject
                    .into_iter()
                    .map(|value| Identity {
                        id: value.url.into(),
                        title: value.title,
                        description: Some(String::from("TODO: Pull from Homosaurus")),
                    })
                    .collect();
                let collections = item_response
                    .item_set
                    .into_iter()
                    .filter_map(|value| {
                        let item_set = item_sets_responses_by_id.get(&value.id)?;

                        Some(Collection {
                            id: item_set.internal_id,
                            title: expect_one(
                                omeka::Property::Title,
                                item_set.title.clone(),
                                item_set.internal_id,
                            )?,
                            description: maybe_one(
                                omeka::Property::Description,
                                item_set.description.clone(),
                                item_set.internal_id,
                            ),
                        })
                    })
                    .collect();
                let files = item_response
                    .media
                    .into_iter()
                    .filter_map(|value| {
                        let media = media_responses_by_id.get(&value.id)?;
                        let filename = expect_one(
                            omeka::Property::Filename,
                            media.filename.clone(),
                            media.internal_id,
                        )?;

                        Some(File {
                            title: expect_one(
                                omeka::Property::Title,
                                media.title.clone(),
                                media.internal_id,
                            )?,
                            filename: filename.clone(),
                            media_type: media.media_type.clone(),
                            url: MediaLocator::Long {
                                slug: slug.clone(),
                                filename,
                            }
                            .to_url()
                            .ok()?
                            .to_string(),
                        })
                    })
                    .collect();

                // Computed properties.
                let start_decade = from_year - (from_year % 10);
                let end_decade = to_year.map(|year| year - (year % 10));
                let decades = end_decade
                    .map(|end_decade| {
                        (start_decade..=end_decade)
                            .step_by(10)
                            .collect::<Vec<u32>>()
                    })
                    .unwrap_or(vec![start_decade]);

                Some(Item {
                    id,
                    slug,
                    title,
                    description,
                    summary,
                    aliases,
                    from_year,
                    to_year,
                    decades,
                    files,
                    links,
                    people,
                    identities,
                    collections,
                })
            })
            .collect())
    }
}
