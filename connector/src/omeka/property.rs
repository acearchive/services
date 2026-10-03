use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InternalId(u32);

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

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
