use std::{collections::HashMap, sync::Arc};

use super::{
    cache,
    models::{Collection, File, Identity, Item, Lang, Link, Person},
    omeka,
};
use crate::url;

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

    pub fn to_url(&self) -> anyhow::Result<reqwest::Url> {
        Ok(match self {
            MediaLocator::Long { slug, filename } => url::file_long(slug, filename)?,
            MediaLocator::Short { id, filename } => url::file_short(id, filename)?,
            MediaLocator::Raw { id, filename } => url::file_raw(id, filename)?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ItemKey {
    ById(omeka::AceId),
    BySlug(omeka::AceSlug),
}

impl From<omeka::AceId> for ItemKey {
    fn from(id: omeka::AceId) -> Self {
        ItemKey::ById(id)
    }
}

impl From<omeka::AceSlug> for ItemKey {
    fn from(slug: omeka::AceSlug) -> Self {
        ItemKey::BySlug(slug)
    }
}

impl From<MediaLocator> for ItemKey {
    fn from(locator: MediaLocator) -> Self {
        match locator {
            MediaLocator::Long { slug, .. } => slug.into(),
            MediaLocator::Short { id, .. } | MediaLocator::Raw { id, .. } => id.into(),
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
    pub canonical_url: CanonicalUrl,
    pub media_type: String,
    pub filename: omeka::AceFilename,
    pub page_url: reqwest::Url,
    pub raw_url: reqwest::Url,
    pub short_url: reqwest::Url,
}

#[derive(Debug)]
pub struct Resolver {
    client: omeka::Client,
}

impl Resolver {
    pub fn new(client: omeka::Client) -> Self {
        Resolver { client }
    }

    /// Resolve Ace Archive identifiers to an Omeka internal item ID.
    ///
    /// Given the ID or slug of an item in the collection, return its internal Omeka ID, its Ace
    /// Archive ID, and its canonical slug.
    async fn resolve_canonical_item(
        &self,
        key: ItemKey,
    ) -> anyhow::Result<(omeka::InternalId, omeka::AceId, omeka::AceSlug)> {
        Ok(match key {
            ItemKey::ById(id) => {
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        &omeka::FindQuery::new().property(omeka::Property::AceId, &id),
                    )
                    .await?;

                let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with property `{}` of `{}`. Using the first one.",
                        omeka::Property::AceId,
                        id,
                    );
                }

                // If there is no matching item, we get an empty array, not a 404 Not Found.
                let item_response = item_responses.first().ok_or(omeka::SkipError)?;

                let canonical_slug =
                    item_response.expect_one(omeka::Property::Slug, |i| &i.slug)?;

                (item_response.internal_id, id, canonical_slug)
            }
            ItemKey::BySlug(slug) => {
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Item,
                        &omeka::FindQuery::new().property(omeka::Property::Slug, &slug),
                    )
                    .await?;

                let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

                if item_responses.len() > 1 {
                    log::warn!(
                        "Multiple items found with property `{}` of `{}`. Using the first one.",
                        omeka::Property::Slug,
                        slug,
                    );
                }

                match item_responses.first() {
                    Some(item_response) => {
                        let id = item_response.expect_one(omeka::Property::AceId, |i| &i.id)?;

                        (item_response.internal_id, id, slug)
                    }
                    None => {
                        let (response, _) = self
                            .client
                            .find(
                                omeka::ResourceType::Item,
                                &omeka::FindQuery::new()
                                    .property(omeka::Property::SlugAlias, &slug),
                            )
                            .await?;

                        let item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

                        if item_responses.len() > 1 {
                            log::warn!(
                                "Multiple items found with property `{}` of `{}`. Using the first one.",
                                omeka::Property::SlugAlias,
                                slug,
                            );
                        }

                        let item_response = item_responses.first().ok_or(omeka::SkipError)?;

                        let id = item_response.expect_one(omeka::Property::AceId, |i| &i.id)?;
                        let canonical_slug =
                            item_response.expect_one(omeka::Property::Slug, |i| &i.slug)?;

                        (item_response.internal_id, id, canonical_slug)
                    }
                }
            }
        })
    }

    /// Resolve Ace Archive identifiers to an Omeka media URL.
    ///
    /// Given the ID or slug of an item in the collection, a filename, and whether this is a "long"
    /// URL, "short" URL, or "raw" URL, return the Omeka URL for the media file.
    ///
    /// If the media resolves via a slug alias and/or a filename alias, also return the file's
    /// canonical URL.
    pub async fn resolve_media(&self, key: MediaLocator) -> anyhow::Result<MediaLocation> {
        if let Some(cached_media_location) = cache::get_media_location(&key) {
            return Ok(cached_media_location);
        }

        let (internal_item_id, id, canonical_slug) =
            self.resolve_canonical_item(key.clone().into()).await?;

        let (response, _) = self
            .client
            .find(
                omeka::ResourceType::Media,
                &omeka::FindQuery::new()
                    .property(omeka::Property::Filename, key.filename())
                    .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
            )
            .await?;

        let media_responses = response.json::<Vec<omeka::MediaResponse>>().await?;

        if media_responses.len() > 1 {
            log::warn!(
                "Multiple media found with property `{}` of `{}`. Using the first one.",
                omeka::Property::Filename,
                key.filename(),
            );
        }

        let media_location = match media_responses.first() {
            Some(media_response) => {
                let canonical_filename =
                    media_response.expect_one(omeka::Property::Filename, |m| &m.filename)?;

                MediaLocation {
                    omeka_url: reqwest::Url::parse(&media_response.original_url)?,
                    canonical_url: match &key {
                        MediaLocator::Long { slug, filename }
                            if slug != &canonical_slug || filename != &canonical_filename =>
                        {
                            CanonicalUrl::ShouldRedirect(
                                MediaLocator::Long {
                                    slug: canonical_slug.clone(),
                                    filename: canonical_filename.clone(),
                                }
                                .to_url()?,
                            )
                        }
                        MediaLocator::Short { .. } => CanonicalUrl::ShouldRedirect(
                            MediaLocator::Long {
                                slug: canonical_slug.clone(),
                                filename: canonical_filename.clone(),
                            }
                            .to_url()?,
                        ),
                        _ => CanonicalUrl::AlreadyCanonical,
                    },
                    media_type: media_response.media_type.clone(),
                    filename: canonical_filename.clone(),
                    page_url: url::artifact_long(&canonical_slug)?,
                    raw_url: MediaLocator::Raw {
                        id: id.clone(),
                        filename: canonical_filename.clone(),
                    }
                    .to_url()?,
                    short_url: MediaLocator::Short {
                        id,
                        filename: canonical_filename,
                    }
                    .to_url()?,
                }
            }
            None => {
                let (response, _) = self
                    .client
                    .find(
                        omeka::ResourceType::Media,
                        &omeka::FindQuery::new()
                            .property(omeka::Property::FilenameAlias, key.filename())
                            .filter(omeka::ResourceFilter::ItemId(internal_item_id)),
                    )
                    .await?;

                let media_responses = response.json::<Vec<omeka::MediaResponse>>().await?;

                if media_responses.len() > 1 {
                    log::warn!(
                        "Multiple media found with property `{}` of `{}`. Using the first one.",
                        omeka::Property::FilenameAlias,
                        key.filename(),
                    );
                }

                let media_response = media_responses.first().ok_or(omeka::SkipError)?;

                let canonical_filename =
                    media_response.expect_one(omeka::Property::Filename, |m| &m.filename)?;

                MediaLocation {
                    omeka_url: reqwest::Url::parse(&media_response.original_url)?,
                    canonical_url: match &key {
                        MediaLocator::Long { .. } | MediaLocator::Short { .. } => {
                            CanonicalUrl::ShouldRedirect(
                                MediaLocator::Long {
                                    slug: canonical_slug.clone(),
                                    filename: canonical_filename.clone(),
                                }
                                .to_url()?,
                            )
                        }
                        MediaLocator::Raw { id, .. } => CanonicalUrl::ShouldRedirect(
                            MediaLocator::Raw {
                                id: id.clone(),
                                filename: canonical_filename.clone(),
                            }
                            .to_url()?,
                        ),
                    },
                    media_type: media_response.media_type.clone(),
                    filename: canonical_filename.clone(),
                    page_url: url::artifact_long(&canonical_slug)?,
                    raw_url: MediaLocator::Raw {
                        id: id.clone(),
                        filename: canonical_filename.clone(),
                    }
                    .to_url()?,
                    short_url: MediaLocator::Short {
                        id,
                        filename: canonical_filename,
                    }
                    .to_url()?,
                }
            }
        };

        cache::put_media_location(key, media_location.clone())?;

        Ok(media_location)
    }

    /// Return metadata about every item in the collection.
    ///
    /// This is cached for a short duration.
    pub async fn list_all_items(&self) -> anyhow::Result<Arc<Vec<Item>>> {
        if let Some(items) = cache::get_items() {
            return Ok(items);
        }

        let _guard = cache::ITEMS_REFRESH_LOCK.lock().await;

        // Another request may have refreshed the cache while we were waiting on the lock.
        if let Some(items) = cache::get_items() {
            return Ok(items);
        }

        let items = Arc::new(self.fetch_all_items().await?);

        cache::put_items(Arc::clone(&items))?;

        Ok(items)
    }

    /// Return metadata about every collection in the archive.
    ///
    /// This is cached for a short duration.
    pub async fn list_all_collections(&self) -> anyhow::Result<Arc<Vec<Collection>>> {
        if let Some(collections) = cache::get_collections() {
            return Ok(collections);
        }

        let _guard = cache::COLLECTIONS_REFRESH_LOCK.lock().await;

        // Another request may have refreshed the cache while we were waiting on the lock.
        if let Some(collections) = cache::get_collections() {
            return Ok(collections);
        }

        let collections = Arc::new(self.fetch_all_collections().await?);

        cache::put_collections(Arc::clone(&collections))?;

        Ok(collections)
    }

    /// Find the item with the given ID.
    pub async fn find_item(&self, id: &omeka::AceId) -> anyhow::Result<Option<Item>> {
        Ok(self
            .list_all_items()
            .await?
            .iter()
            .find(|item| &item.id == id)
            .cloned())
    }

    /// Find the file with the given filename in the item with the given ID.
    pub async fn find_file(
        &self,
        id: &omeka::AceId,
        filename: &omeka::AceFilename,
    ) -> anyhow::Result<Option<File>> {
        Ok(self.find_item(id).await?.and_then(|item| {
            item.files
                .into_iter()
                .find(|file| &file.filename == filename)
        }))
    }

    /// Find the person with the given name, along with every item they created.
    pub async fn find_person(&self, name: &str) -> anyhow::Result<Option<(Person, Vec<Item>)>> {
        let items = self.list_all_items().await?;

        let Some(person) = items
            .iter()
            .flat_map(|item| &item.people)
            .find(|person| person.name == name)
            .cloned()
        else {
            return Ok(None);
        };

        let creations = items
            .iter()
            .filter(|item| item.people.iter().any(|p| p.name == person.name))
            .cloned()
            .collect();

        Ok(Some((person, creations)))
    }

    /// Find the identity with the given name, along with every item about it.
    pub async fn find_identity(&self, name: &str) -> anyhow::Result<Option<(Identity, Vec<Item>)>> {
        let items = self.list_all_items().await?;

        let Some(identity) = items
            .iter()
            .flat_map(|item| &item.identities)
            .find(|identity| identity.name.value == name)
            .cloned()
        else {
            return Ok(None);
        };

        let subjects = items
            .iter()
            .filter(|item| {
                item.identities
                    .iter()
                    .any(|i| i.name.value == identity.name.value)
            })
            .cloned()
            .collect();

        Ok(Some((identity, subjects)))
    }

    /// Find the collection with the given title, along with every item in it.
    pub async fn find_collection(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<(Collection, Vec<Item>)>> {
        let Some(collection) = self
            .list_all_collections()
            .await?
            .iter()
            .find(|collection| collection.title.value == name)
            .cloned()
        else {
            return Ok(None);
        };

        let members = self
            .list_all_items()
            .await?
            .iter()
            .filter(|item| {
                item.collections
                    .iter()
                    .any(|c| c.title.value == collection.title.value)
            })
            .cloned()
            .collect();

        Ok(Some((collection, members)))
    }

    async fn fetch_all_items(&self) -> anyhow::Result<Vec<Item>> {
        // Given the size of the archive, paginating through all items, item sets, and media and
        // collecting them in memory is going to be much faster than the alternative.
        //
        // TODO: Can these API calls be made concurrent?
        let (mut response, mut maybe_next_page) = self
            .client
            .find(
                omeka::ResourceType::Item,
                &omeka::FindQuery::new()
                    .has_property(omeka::Property::AceId)
                    .has_property(omeka::Property::Slug),
            )
            .await?;

        let mut item_responses = response.json::<Vec<omeka::ItemResponse>>().await?;

        while let Some(page) = maybe_next_page {
            (response, maybe_next_page) = self.client.next_page(page).await?;
            item_responses.extend(response.json::<Vec<omeka::ItemResponse>>().await?);
        }

        let (mut response, mut maybe_next_page) = self
            .client
            .find(omeka::ResourceType::ItemSet, &omeka::FindQuery::new())
            .await?;

        let mut item_set_responses = response.json::<Vec<omeka::ItemSetResponse>>().await?;

        while let Some(page) = maybe_next_page {
            (response, maybe_next_page) = self.client.next_page(page).await?;
            item_set_responses.extend(response.json::<Vec<omeka::ItemSetResponse>>().await?);
        }

        let (mut response, mut maybe_next_page) = self
            .client
            .find(omeka::ResourceType::Media, &omeka::FindQuery::new())
            .await?;

        let mut media_responses = response.json::<Vec<omeka::MediaResponse>>().await?;

        while let Some(page) = maybe_next_page {
            (response, maybe_next_page) = self.client.next_page(page).await?;
            media_responses.extend(response.json::<Vec<omeka::MediaResponse>>().await?);
        }

        let item_set_responses_by_id: HashMap<omeka::InternalId, omeka::ItemSetResponse> =
            item_set_responses
                .into_iter()
                .map(|item_set_response| (item_set_response.internal_id, item_set_response))
                .collect();

        let media_responses_by_id: HashMap<omeka::InternalId, omeka::MediaResponse> =
            media_responses
                .into_iter()
                .map(|media_response| (media_response.internal_id, media_response))
                .collect();

        Ok(item_responses
            .into_iter()
            .filter_map(|item_response| {
                // Required properties.
                let id = item_response
                    .expect_one(omeka::Property::AceId, |i| &i.id)
                    .ok()?;
                let slug = item_response
                    .expect_one(omeka::Property::Slug, |i| &i.slug)
                    .ok()?;
                let title = item_response
                    .expect_localized(omeka::Property::Title, |i| &i.title)
                    .ok()?;
                let summary = item_response
                    .expect_localized(omeka::Property::Abstract, |i| &i.summary)
                    .ok()?;
                let (from_year, to_year) = item_response
                    .expect_one(omeka::Property::Created, |i| &i.created)
                    .ok()?
                    .split_once('/')
                    .map(|(from, to)| (from.parse::<u32>().ok(), to.parse::<u32>().ok()))
                    .unwrap_or((None, None));
                let from_year = from_year?;

                // Optional single-value properties.
                let description = item_response
                    .maybe_localized(omeka::Property::Description, |i| &i.description);
                let schema_class = item_response.schema_class();

                // Multi-value properties.
                let people = item_response
                    .creator
                    .into_iter()
                    .map(|value| Person { name: value.title })
                    .collect();
                let links = item_response
                    .relation
                    .into_iter()
                    .filter_map(|value| {
                        let url = reqwest::Url::parse(&value.url)
                            .inspect_err(|err| {
                                log::warn!(
                                    "Item with internal ID `{}` has an invalid relation URL `{}`: {}",
                                    item_response.internal_id,
                                    value.url,
                                    err,
                                );
                            })
                            .ok()?;

                        Some(Link {
                            title: value.localized_title(),
                            url,
                        })
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
                    .map(|value| match value {
                        omeka::SuggestedPropertyValue::Literal(value) => Identity {
                            name: value.into(),
                            url: None,
                        },
                        omeka::SuggestedPropertyValue::Url(value) => Identity {
                            name: value.localized_title(),
                            url: reqwest::Url::parse(&value.url)
                                .inspect_err(|err| {
                                    log::warn!(
                                        "Item with internal ID `{}` has an invalid subject URL `{}`: {}",
                                        item_response.internal_id,
                                        value.url,
                                        err,
                                    );
                                })
                                .ok(),
                        },
                    })
                    .collect();
                let collections = item_response
                    .item_set
                    .into_iter()
                    .filter_map(|value| {
                        let item_set_response = item_set_responses_by_id.get(&value.id)?;

                        Some(Collection {
                            schema_class: item_set_response.schema_class(),
                            title: item_set_response
                                .expect_localized(omeka::Property::Title, |i| &i.title)
                                .ok()?,
                            description: item_set_response
                                .maybe_localized(omeka::Property::Description, |i| &i.description),
                        })
                    })
                    .collect();
                let files = item_response
                    .media
                    .into_iter()
                    .filter_map(|value| {
                        let media = media_responses_by_id.get(&value.id)?;
                        let filename =
                            media.maybe_one(omeka::Property::Filename, |m| &m.filename)?;
                        let omeka_url = reqwest::Url::parse(&media.original_url)
                            .inspect_err(|err| {
                                log::warn!(
                                    "Media with internal ID `{}` has an invalid original URL `{}`: {}",
                                    media.internal_id,
                                    media.original_url,
                                    err,
                                );
                            })
                            .ok()?;

                        Some(File {
                            title: media.maybe_localized(omeka::Property::Title, |m| &m.title)?,
                            filename,
                            schema_class: media.schema_class(),
                            media_type: media.media_type.clone(),
                            lang: media.lang.clone().map(Lang::new),
                            omeka_url,
                        })
                    })
                    .collect();

                Some(Item {
                    id,
                    slug,
                    schema_class,
                    title,
                    description,
                    summary,
                    aliases,
                    from_year,
                    to_year,
                    files,
                    links,
                    people,
                    identities,
                    collections,
                })
            })
            .collect())
    }

    async fn fetch_all_collections(&self) -> anyhow::Result<Vec<Collection>> {
        let (mut response, mut maybe_next_page) = self
            .client
            .find(omeka::ResourceType::ItemSet, &omeka::FindQuery::new())
            .await?;

        let mut item_set_responses = response.json::<Vec<omeka::ItemSetResponse>>().await?;

        while let Some(page) = maybe_next_page {
            (response, maybe_next_page) = self.client.next_page(page).await?;
            item_set_responses.extend(response.json::<Vec<omeka::ItemSetResponse>>().await?);
        }

        Ok(item_set_responses
            .into_iter()
            .filter_map(|item_set_response| {
                Some(Collection {
                    schema_class: item_set_response.schema_class(),
                    title: item_set_response
                        .expect_localized(omeka::Property::Title, |i| &i.title)
                        .ok()?,
                    description: item_set_response
                        .maybe_localized(omeka::Property::Description, |i| &i.description),
                })
            })
            .collect())
    }
}
