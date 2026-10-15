use std::fmt;

use serde::Deserialize;

use super::property::{AceFilename, AceId, AceSlug, InternalId, Property, ResourceType};
use crate::models::{Lang, LocalizedString, SchemaClass};

/// An error signaling that a resource should be skipped because it is missing required data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkipError;

impl fmt::Display for SkipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SkipError")
    }
}

impl std::error::Error for SkipError {}

impl ResourceType {
    fn label(&self) -> &'static str {
        match self {
            ResourceType::Item => "Item",
            ResourceType::ItemSet => "Item set",
            ResourceType::Media => "Media",
        }
    }
}

fn expect_one<T>(
    resource_type: ResourceType,
    internal_id: InternalId,
    property: Property,
    values: &[LiteralPropertyValue<T>],
) -> anyhow::Result<LiteralPropertyValue<T>>
where
    T: fmt::Display + Clone,
{
    let resource_type_label = resource_type.label();

    if values.len() > 1 {
        let value = values.first().unwrap();

        log::warn!(
            "{} with internal ID `{}` has multiple values for property `{}`. Using the first one: {}",
            resource_type_label,
            internal_id,
            property,
            value.value,
        );
    }

    match values.first().cloned() {
        Some(value) => Ok(value),
        None => {
            log::warn!(
                "{} with internal ID `{}` is missing required property `{}`.",
                resource_type_label,
                internal_id,
                property
            );

            Err(SkipError.into())
        }
    }
}

fn maybe_one<T>(
    resource_type: ResourceType,
    internal_id: InternalId,
    property: Property,
    values: &[LiteralPropertyValue<T>],
) -> Option<LiteralPropertyValue<T>>
where
    T: fmt::Display + Clone,
{
    let resource_type_label = resource_type.label();

    if values.len() > 1 {
        let value = values.first().unwrap();

        log::warn!(
            "{} with internal ID `{}` has multiple values for property `{}`. Using the first one: {}",
            resource_type_label,
            internal_id,
            property,
            value.value,
        );
    }

    values.first().cloned()
}

fn schema_class(
    resource_type: ResourceType,
    internal_id: InternalId,
    json_ld_type: &JsonLdType,
) -> Option<SchemaClass> {
    let Some(term) = json_ld_type.resource_class() else {
        log::warn!(
            "{} with internal ID `{}` has no resource class.",
            resource_type.label(),
            internal_id,
        );

        return None;
    };

    let schema_class = term.strip_prefix("schema:").map(SchemaClass::new);

    if schema_class.is_none() {
        log::warn!(
            "{} with internal ID `{}` has resource class `{}`, which is not a Schema.org class. Ignoring it.",
            resource_type.label(),
            internal_id,
            term,
        );
    }

    schema_class
}

/// The JSON-LD `@type` of an Omeka resource.
///
/// Omeka always includes its own type (e.g. `o:Item`). If the resource has a resource class, that
/// is included as well, in which case this is an array rather than a single value.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonLdType {
    One(String),
    Many(Vec<String>),
}

impl JsonLdType {
    /// Return the term of the resource class (e.g. `schema:CreativeWork`), if there is one.
    pub fn resource_class(&self) -> Option<&str> {
        let types = match self {
            JsonLdType::One(term) => std::slice::from_ref(term),
            JsonLdType::Many(terms) => terms.as_slice(),
        };

        types
            .iter()
            .map(String::as_str)
            .find(|term| !term.starts_with("o:"))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteralPropertyValue<T> {
    #[serde(rename = "@value")]
    pub value: T,

    #[serde(rename = "@language")]
    pub lang: Option<String>,
}

impl From<LiteralPropertyValue<String>> for LocalizedString {
    fn from(value: LiteralPropertyValue<String>) -> Self {
        LocalizedString {
            value: value.value,
            lang: value.lang.map(Lang::new),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UrlPropertyValue {
    #[serde(rename = "@id")]
    pub url: String,

    #[serde(rename = "o:label")]
    pub title: String,

    /// The language of the label, not of the resource the URL points to.
    #[serde(rename = "@language")]
    pub lang: Option<String>,
}

impl UrlPropertyValue {
    pub fn localized_title(&self) -> LocalizedString {
        LocalizedString {
            value: self.title.clone(),
            lang: self.lang.clone().map(Lang::new),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResourcePropertyValue {
    #[serde(rename = "display_title")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct IdPropertyValue {
    #[serde(rename = "o:id")]
    pub id: InternalId,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SuggestedPropertyValue {
    Url(UrlPropertyValue),
    Literal(LiteralPropertyValue<String>),
}

#[derive(Debug, Deserialize)]
pub struct ItemResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(rename = "@type")]
    pub json_ld_type: JsonLdType,

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
    pub subject: Vec<SuggestedPropertyValue>,

    #[serde(default, rename = "dcterms:relation")]
    pub relation: Vec<UrlPropertyValue>,

    #[serde(default, rename = "o:media")]
    pub media: Vec<IdPropertyValue>,

    #[serde(default, rename = "o:item_set")]
    pub item_set: Vec<IdPropertyValue>,
}

impl ItemResponse {
    pub fn schema_class(&self) -> Option<SchemaClass> {
        schema_class(ResourceType::Item, self.internal_id, &self.json_ld_type)
    }

    pub fn expect_one<T, F>(&self, property: Property, f: F) -> anyhow::Result<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&ItemResponse) -> &[LiteralPropertyValue<T>],
    {
        expect_one(ResourceType::Item, self.internal_id, property, f(self)).map(|v| v.value)
    }

    pub fn expect_localized<F>(&self, property: Property, f: F) -> anyhow::Result<LocalizedString>
    where
        F: FnOnce(&ItemResponse) -> &[LiteralPropertyValue<String>],
    {
        expect_one(ResourceType::Item, self.internal_id, property, f(self)).map(Into::into)
    }

    pub fn maybe_localized<F>(&self, property: Property, f: F) -> Option<LocalizedString>
    where
        F: FnOnce(&ItemResponse) -> &[LiteralPropertyValue<String>],
    {
        maybe_one(ResourceType::Item, self.internal_id, property, f(self)).map(Into::into)
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemSetResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(rename = "@type")]
    pub json_ld_type: JsonLdType,

    #[serde(rename = "dcterms:title")]
    pub title: Vec<LiteralPropertyValue<String>>,

    #[serde(rename = "dcterms:description")]
    pub description: Vec<LiteralPropertyValue<String>>,
}

impl ItemSetResponse {
    pub fn schema_class(&self) -> Option<SchemaClass> {
        schema_class(ResourceType::ItemSet, self.internal_id, &self.json_ld_type)
    }

    pub fn expect_localized<F>(&self, property: Property, f: F) -> anyhow::Result<LocalizedString>
    where
        F: FnOnce(&ItemSetResponse) -> &[LiteralPropertyValue<String>],
    {
        expect_one(ResourceType::ItemSet, self.internal_id, property, f(self)).map(Into::into)
    }

    pub fn maybe_localized<F>(&self, property: Property, f: F) -> Option<LocalizedString>
    where
        F: FnOnce(&ItemSetResponse) -> &[LiteralPropertyValue<String>],
    {
        maybe_one(ResourceType::ItemSet, self.internal_id, property, f(self)).map(Into::into)
    }
}

#[derive(Debug, Deserialize)]
pub struct MediaResponse {
    #[serde(rename = "o:id")]
    pub internal_id: InternalId,

    #[serde(rename = "@type")]
    pub json_ld_type: JsonLdType,

    #[serde(rename = "o:original_url")]
    pub original_url: String,

    #[serde(rename = "o:media_type")]
    pub media_type: String,

    #[serde(rename = "o:lang")]
    pub lang: Option<String>,

    #[serde(rename = "dcterms:title")]
    pub title: Vec<LiteralPropertyValue<String>>,

    #[serde(rename = "ace:filename")]
    pub filename: Vec<LiteralPropertyValue<AceFilename>>,
}

impl MediaResponse {
    pub fn schema_class(&self) -> Option<SchemaClass> {
        schema_class(ResourceType::Media, self.internal_id, &self.json_ld_type)
    }

    pub fn expect_one<T, F>(&self, property: Property, f: F) -> anyhow::Result<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&MediaResponse) -> &[LiteralPropertyValue<T>],
    {
        expect_one(ResourceType::Media, self.internal_id, property, f(self)).map(|v| v.value)
    }

    pub fn maybe_one<T, F>(&self, property: Property, f: F) -> Option<T>
    where
        T: fmt::Display + Clone,
        F: FnOnce(&MediaResponse) -> &[LiteralPropertyValue<T>],
    {
        maybe_one(ResourceType::Media, self.internal_id, property, f(self)).map(|v| v.value)
    }

    pub fn maybe_localized<F>(&self, property: Property, f: F) -> Option<LocalizedString>
    where
        F: FnOnce(&MediaResponse) -> &[LiteralPropertyValue<String>],
    {
        maybe_one(ResourceType::Media, self.internal_id, property, f(self)).map(Into::into)
    }
}
