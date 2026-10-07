use std::collections::HashMap;

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

fn page_url(slug: &omeka::AceSlug) -> anyhow::Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "https://{}/artifacts/{}",
        config::base_domain()?,
        slug.as_ref()
    ))?)
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
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
                        omeka::Property::Slug,
                        slug,
                    );
                }

                match item_responses.first() {
                    Some(item_response) => {
                        let canonical_slug =
                            item_response.expect_one(omeka::Property::Slug, |i| &i.slug)?;
                        let id = item_response.expect_one(omeka::Property::AceId, |i| &i.id)?;

                        (item_response.internal_id, id, canonical_slug)
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
                                "Multiple items found with `{}` property value `{}`. Using the first one.",
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
                "Multiple items found with `{}` property value `{}`. Using the first one.",
                omeka::Property::Filename,
                key.filename(),
            );
        }

        let media_location = match media_responses.first() {
            Some(media_response) => {
                let filename =
                    media_response.expect_one(omeka::Property::Filename, |m| &m.filename)?;

                MediaLocation {
                    omeka_url: reqwest::Url::parse(&media_response.original_url)?,
                    canonical_url: match &key {
                        MediaLocator::Long { slug, .. } if slug != &canonical_slug => {
                            CanonicalUrl::ShouldRedirect(
                                key.clone()
                                    .with_key(canonical_slug.clone().into())
                                    .to_url()?,
                            )
                        }
                        _ => CanonicalUrl::AlreadyCanonical,
                    },
                    media_type: media_response.media_type.clone(),
                    filename: filename.clone(),
                    page_url: page_url(&canonical_slug)?,
                    raw_url: MediaLocator::Raw {
                        id: id.clone(),
                        filename: filename.clone(),
                    }
                    .to_url()?,
                    short_url: MediaLocator::Short { id, filename }.to_url()?,
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
                        "Multiple items found with `{}` property value `{}`. Using the first one.",
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
                        MediaLocator::Long { .. } => CanonicalUrl::ShouldRedirect(
                            key.clone()
                                .with_key_and_filename(
                                    canonical_slug.clone().into(),
                                    canonical_filename.clone(),
                                )
                                .to_url()?,
                        ),
                        MediaLocator::Short { id, .. } | MediaLocator::Raw { id, .. } => {
                            CanonicalUrl::ShouldRedirect(
                                key.clone()
                                    .with_key_and_filename(
                                        id.clone().into(),
                                        canonical_filename.clone(),
                                    )
                                    .to_url()?,
                            )
                        }
                    },
                    media_type: media_response.media_type.clone(),
                    filename: canonical_filename.clone(),
                    page_url: page_url(&canonical_slug)?,
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

        cache::put_media_location(key, media_location.clone());

        Ok(media_location)
    }

    /// Return metadata about every item in the collection.
    pub async fn list_items(&self) -> anyhow::Result<Vec<Item>> {
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
                let id = item_response.maybe_one(omeka::Property::AceId, |i| &i.id)?;
                let slug = item_response.maybe_one(omeka::Property::Slug, |i| &i.slug)?;
                let title = item_response.maybe_one(omeka::Property::Title, |i| &i.title)?;
                let summary = item_response.maybe_one(omeka::Property::Abstract, |i| &i.summary)?;
                let (from_year, to_year) = item_response
                    .maybe_one(omeka::Property::Created, |i| &i.created)?
                    .split_once('/')
                    .map(|(from, to)| (from.parse::<u32>().ok(), to.parse::<u32>().ok()))
                    .unwrap_or((None, None));
                let from_year = from_year?;

                // Optional single-value properties.
                let description =
                    item_response.maybe_one(omeka::Property::Description, |i| &i.description);

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
                    })
                    .collect();
                let collections = item_response
                    .item_set
                    .into_iter()
                    .filter_map(|value| {
                        let item_set = item_set_responses_by_id.get(&value.id)?;

                        Some(Collection {
                            id: item_set.internal_id,
                            title: item_set.maybe_one(omeka::Property::Title, |i| &i.title)?,
                            description: item_set
                                .maybe_one(omeka::Property::Description, |i| &i.description),
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

                        Some(File {
                            title: media.maybe_one(omeka::Property::Title, |m| &m.title)?,
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
