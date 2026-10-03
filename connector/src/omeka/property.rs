use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum Property {
    Id,
    Slug,
    SlugAlias,
    Filename,
    FilenameAlias,
}

impl Property {
    pub fn as_str(&self) -> &'static str {
        match self {
            Property::Id => "ace:id",
            Property::Slug => "ace:slug",
            Property::SlugAlias => "ace:slugAlias",
            Property::Filename => "ace:filename",
            Property::FilenameAlias => "ace:filenameAlias",
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
