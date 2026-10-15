use serde::Serialize;

use super::omeka::{AceFilename, AceId, AceSlug};

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
