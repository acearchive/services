use serde::Serialize;

use crate::omeka::InternalId;

use super::omeka::{AceFilename, AceId, AceSlug, ExternalId};

#[derive(Debug, Serialize)]
pub struct File {
    pub title: String,
    pub filename: AceFilename,
    pub media_type: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct Person {
    pub id: InternalId,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct Identity {
    pub id: ExternalId,
    pub title: String,
    // TODO: Pull a description from Homosaurus.
    // pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Collection {
    pub id: InternalId,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub id: AceId,
    pub slug: AceSlug,
    pub aliases: Vec<AceSlug>,
    pub title: String,
    #[serde(rename = "abstract")]
    pub summary: String,
    pub description: Option<String>,
    pub from_year: u32,
    pub to_year: Option<u32>,
    pub decades: Vec<u32>,
    pub files: Vec<File>,
    pub links: Vec<Link>,
    pub people: Vec<Person>,
    pub identities: Vec<Identity>,
    pub collections: Vec<Collection>,
}
