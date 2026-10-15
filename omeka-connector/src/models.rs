use std::{borrow::Cow, fmt};

use super::omeka::{AceFilename, AceId, AceSlug};

#[derive(Debug, Clone)]
pub struct Lang(Cow<'static, str>);

impl Lang {
    pub fn new<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Lang(s.into())
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct LocalizedString {
    pub value: String,
    pub lang: Option<Lang>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub title: LocalizedString,
    pub filename: AceFilename,
    pub media_type: String,
    // The language of the file's contents, which may differ from the language of its title.
    pub lang: Option<Lang>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub title: LocalizedString,
    pub url: reqwest::Url,
}

#[derive(Debug, Clone)]
pub struct Person {
    // Names of people are not localized.
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub name: LocalizedString,
    // An identity may come from a controlled vocabulary (e.g. Homosaurus), in which case it has a
    // URL, or it may be a free-text label.
    pub url: Option<reqwest::Url>,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub title: LocalizedString,
    pub description: Option<LocalizedString>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: AceId,
    pub slug: AceSlug,
    pub aliases: Vec<AceSlug>,
    pub title: LocalizedString,
    pub summary: LocalizedString,
    pub description: Option<LocalizedString>,
    pub from_year: u32,
    pub to_year: Option<u32>,
    pub files: Vec<File>,
    pub links: Vec<Link>,
    pub people: Vec<Person>,
    pub identities: Vec<Identity>,
    pub collections: Vec<Collection>,
}
