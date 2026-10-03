use std::fmt;

use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize)]
pub struct InternalId(u32);

impl fmt::Display for InternalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct AceId(String);

impl AsRef<str> for AceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct AceSlug(String);

impl AsRef<str> for AceSlug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AceSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AceFilename(String);

impl AsRef<str> for AceFilename {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AceFilename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    Item,
    ItemSet,
    Media,
}

impl ResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Item => "items",
            ResourceType::ItemSet => "item_sets",
            ResourceType::Media => "media",
        }
    }
}

impl AsRef<str> for ResourceType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Property {
    InternalId,
    AceId,
    AceSlug,
    AceSlugAlias,
    AceFilename,
    AceFilenameAlias,
}

impl Property {
    pub fn as_str(&self) -> &'static str {
        // We're hardcoding the namespace prefixes here. Technically we should be querying the
        // JSON-LD `@context` field to look them up, but this is fine for now. They won't change
        // unless we change them.
        match self {
            Property::InternalId => "o:id",
            Property::AceId => "ace:id",
            Property::AceSlug => "ace:slug",
            Property::AceSlugAlias => "ace:slugAlias",
            Property::AceFilename => "ace:filename",
            Property::AceFilenameAlias => "ace:filenameAlias",
        }
    }
}

impl AsRef<str> for Property {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub enum ResourceFilter {
    ItemId(InternalId),
}

impl ResourceFilter {
    pub fn as_query_param(&self) -> (String, String) {
        match self {
            ResourceFilter::ItemId(id) => (String::from("item_id"), id.0.to_string()),
        }
    }
}
