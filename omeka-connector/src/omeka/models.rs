use std::fmt;

use serde::Deserialize;

use super::property::{AceFilename, AceId, AceSlug, InternalId, Property, ResourceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipError;

impl fmt::Display for SkipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipError")
    }
}

impl std::error::Error for SkipError {}

fn expect_one<T>(
    resource_type: ResourceType,
    internal_id: InternalId,
    property: Property,
    values: &Vec<LiteralPropertyValue<T>>,
) -> anyhow::Result<T>
where
    T: fmt::Display + Clone,
{
    let resource_type_label = match resource_type {
        ResourceType::Item => "Item",
        ResourceType::ItemSet => "Item set",
        ResourceType::Media => "Media",
    };

    if values.len() > 1 {
        let value = values.first().unwrap();

        log::warn!(
            "{} with internal ID `{}` has multiple values for property `{}`. Using the first one: {}",
            resource_type_label,
            internal_id,
            &property,
            value.value,
        );
    }

    match values.into_iter().next().map(|value| value.value.clone()) {
        Some(value) => Ok(value),
        None => {
            log::warn!(
                "{} with internal ID `{}` is missing required property `{}`.",
                resource_type_label,
                internal_id,
                &property
            );

            Err(SkipError.into())
        }
    }
}

fn maybe_one<T>(
    resource_type: ResourceType,
    internal_id: InternalId,
    property: Property,
    values: &Vec<LiteralPropertyValue<T>>,
) -> Option<T>
where
    T: fmt::Display + Clone,
{
    let resource_type_label = match resource_type {
        ResourceType::Item => "Item",
        ResourceType::ItemSet => "Item set",
        ResourceType::Media => "Media",
    };

    if values.len() > 1 {
        let value = values.first().unwrap();

        log::warn!(
            "{} with internal ID `{}` has multiple values for property `{}`. Using the first one: {}",
            resource_type_label,
            internal_id,
            &property,
            value.value,
        );
    }

    values.first().map(|value| value.value.clone()).clone()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteralPropertyValue<T> {
    #[serde(rename = "@value")]
    pub value: T,
    pub property_label: String,
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

impl ItemResponse {
    pub fn expect_one<T, F>(&self, property: Property, f: F) -> anyhow::Result<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&ItemResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        expect_one(ResourceType::Item, self.internal_id, property, f(self))
    }

    pub fn maybe_one<T, F>(&self, property: Property, f: F) -> Option<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&ItemResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        maybe_one(ResourceType::Item, self.internal_id, property, f(self))
    }
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

impl ItemSetResponse {
    pub fn expect_one<T, F>(&self, property: Property, f: F) -> anyhow::Result<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&ItemSetResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        expect_one(ResourceType::ItemSet, self.internal_id, property, f(self))
    }

    pub fn maybe_one<T, F>(&self, property: Property, f: F) -> Option<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&ItemSetResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        maybe_one(ResourceType::ItemSet, self.internal_id, property, f(self))
    }
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

impl MediaResponse {
    pub fn expect_one<T, F>(&self, property: Property, f: F) -> anyhow::Result<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&MediaResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        expect_one(ResourceType::Media, self.internal_id, property, f(self))
    }

    pub fn maybe_one<T, F>(&self, property: Property, f: F) -> Option<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&MediaResponse) -> &Vec<LiteralPropertyValue<T>>,
    {
        maybe_one(ResourceType::Media, self.internal_id, property, f(self))
    }
}
