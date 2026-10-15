use serde::Serialize;

use crate::{
    models,
    omeka::{AceFilename, AceId, AceSlug},
    url,
};

#[derive(Debug, Serialize)]
pub struct File {
    pub title: String,
    pub filename: AceFilename,
    pub media_type: String,
    pub lang: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub id: AceId,
    pub slug: AceSlug,
    pub aliases: Vec<AceSlug>,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    pub from_year: u32,
    pub to_year: Option<u32>,
    pub decades: Vec<u32>,
    pub files: Vec<File>,
    pub links: Vec<Link>,
    pub people: Vec<String>,
    pub identities: Vec<String>,
    pub collections: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TagKind {
    Collection,
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub name: String,
    pub kind: TagKind,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CollectionMetadata {
    pub tags: Vec<Tag>,
}

impl File {
    fn from_model(file: models::File, slug: &AceSlug) -> anyhow::Result<Self> {
        Ok(File {
            url: url::file_long(slug, &file.filename)?.to_string(),
            title: file.title.value,
            filename: file.filename,
            media_type: file.media_type,
            lang: file.lang.map(|lang| lang.to_string()),
        })
    }
}

impl From<models::Link> for Link {
    fn from(link: models::Link) -> Self {
        Link {
            title: link.title.value,
            url: link.url.to_string(),
        }
    }
}

impl TryFrom<models::Item> for Item {
    type Error = anyhow::Error;

    fn try_from(item: models::Item) -> anyhow::Result<Self> {
        let start_decade = item.from_year - (item.from_year % 10);
        let end_decade = item.to_year.map(|year| year - (year % 10));
        let decades = end_decade
            .map(|end_decade| {
                (start_decade..=end_decade)
                    .step_by(10)
                    .collect::<Vec<u32>>()
            })
            .unwrap_or(vec![start_decade]);

        let files = item
            .files
            .into_iter()
            .map(|file| File::from_model(file, &item.slug))
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Item {
            id: item.id,
            slug: item.slug,
            aliases: item.aliases,
            title: item.title.value,
            summary: item.summary.value,
            description: item.description.map(|description| description.value),
            from_year: item.from_year,
            to_year: item.to_year,
            decades,
            files,
            links: item.links.into_iter().map(Link::from).collect(),
            people: item.people.into_iter().map(|person| person.name).collect(),
            identities: item
                .identities
                .into_iter()
                .map(|identity| identity.name.value)
                .collect(),
            collections: item
                .collections
                .into_iter()
                .map(|collection| collection.title.value)
                .collect(),
        })
    }
}

impl From<models::Collection> for Tag {
    fn from(collection: models::Collection) -> Self {
        Tag {
            name: collection.title.value,
            kind: TagKind::Collection,
            description: collection.description.map(|description| description.value),
        }
    }
}
