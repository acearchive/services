use serde::Deserialize;

use super::property::{AceFilename, AceId, AceSlug, InternalId};

#[derive(Debug, Clone, Deserialize)]
pub struct LiteralPropertyValue<T> {
    #[serde(rename = "@value")]
    pub value: T,
}

#[derive(Debug, Deserialize)]
pub struct UrlPropertyValue {
    #[serde(rename = "@id")]
    pub url: String,

    #[serde(rename = "o:label")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ResourcePropertyValue {
    #[serde(rename = "value_resource_id")]
    pub id: InternalId,

    #[serde(rename = "display_title")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct IdPropertyValue {
    #[serde(rename = "o:id")]
    pub id: InternalId,
}

#[derive(Debug, Deserialize)]
pub struct ItemResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(default, rename = "ace:id")]
    pub id: Vec<LiteralPropertyValue<AceId>>,

    #[serde(default, rename = "ace:slug")]
    pub slug: Vec<LiteralPropertyValue<AceSlug>>,

    #[serde(default, rename = "ace:slugAlias")]
    pub slug_alias: Vec<LiteralPropertyValue<AceSlug>>,

    #[serde(default, rename = "dcterms:title")]
    pub title: Vec<LiteralPropertyValue<String>>,

    #[serde(default, rename = "dcterms:description")]
    pub description: Vec<LiteralPropertyValue<String>>,

    #[serde(default, rename = "dcterms:abstract")]
    pub summary: Vec<LiteralPropertyValue<String>>,

    #[serde(default, rename = "dcterms:created")]
    pub created: Vec<LiteralPropertyValue<String>>,

    #[serde(default, rename = "dcterms:creator")]
    pub creator: Vec<ResourcePropertyValue>,

    #[serde(default, rename = "dcterms:subject")]
    pub subject: Vec<UrlPropertyValue>,

    #[serde(default, rename = "dcterms:relation")]
    pub relation: Vec<UrlPropertyValue>,

    #[serde(default, rename = "o:media")]
    pub media: Vec<IdPropertyValue>,

    #[serde(default, rename = "o:item_set")]
    pub item_set: Vec<IdPropertyValue>,
}

#[derive(Debug, Deserialize)]
pub struct ItemSetResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(rename = "dcterms:title")]
    pub title: Vec<LiteralPropertyValue<String>>,

    #[serde(rename = "dcterms:description")]
    pub description: Vec<LiteralPropertyValue<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MediaResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(rename = "o:original_url")]
    pub original_url: String,

    #[serde(rename = "o:media_type")]
    pub media_type: String,

    #[serde(rename = "dcterms:title")]
    pub title: Vec<LiteralPropertyValue<String>>,

    #[serde(rename = "ace:filename")]
    pub filename: Vec<LiteralPropertyValue<AceFilename>>,
}
