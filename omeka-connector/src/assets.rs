use std::{fmt, sync::LazyLock};

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{config, models, resolver::MediaLocation};

static TEMPLATES: LazyLock<tera::Tera> = LazyLock::new(|| {
    let mut tera = tera::Tera::default();

    let template = include_str!("assets/base.html");
    tera.add_raw_template("base", template)
        .expect("Failed to add base template.");

    let template = include_str!("assets/image.html");
    tera.add_raw_template("image", template)
        .expect("Failed to add image template.");

    let template = include_str!("assets/video.html");
    tera.add_raw_template("video", template)
        .expect("Failed to add video template.");

    let template = include_str!("assets/pdf.html");
    tera.add_raw_template("pdf", template)
        .expect("Failed to add PDF template.");

    let template = include_str!("assets/page.html");
    tera.add_raw_template("page", template)
        .expect("Failed to add page template.");

    let template = include_str!("assets/style.css");
    tera.add_raw_template("style", template)
        .expect("Failed to add style template.");

    let template = include_str!("assets/export-readme.txt");
    tera.add_raw_template("export-readme", template)
        .expect("Failed to add export README template.");

    let template = include_str!("assets/item-readme.md");
    tera.add_raw_template("item-readme", template)
        .expect("Failed to add item README template.");

    tera
});

#[derive(Debug, Clone)]
pub struct FilePageContext {
    pub title: String,
    pub base_domain: String,
    pub page_url: String,
    pub raw_url: String,
    pub short_url: String,
    pub media_type: String,
}

impl FilePageContext {
    pub fn from_location(location: &MediaLocation) -> Result<Self, anyhow::Error> {
        Ok(Self {
            title: location.filename.to_string(),
            base_domain: config::base_domain().to_owned(),
            page_url: location.page_url.to_string(),
            raw_url: location.raw_url.to_string(),
            short_url: location.short_url.to_string(),
            media_type: location.media_type.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilePage(String);

impl fmt::Display for FilePage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FilePage {
    pub fn render(page_context: &FilePageContext) -> Option<Self> {
        let fragment = if page_context.media_type.starts_with("image/") {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url", &page_context.raw_url);

            Some(
                TEMPLATES
                    .render("image", &template_context)
                    .expect("Failed to render image template."),
            )
        } else if page_context.media_type.starts_with("video/") {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url", &page_context.raw_url);
            template_context.insert("media_type", &page_context.media_type);

            Some(
                TEMPLATES
                    .render("video", &template_context)
                    .expect("Failed to render video template."),
            )
        } else if page_context.media_type == "application/pdf" {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url", &page_context.raw_url);

            Some(
                TEMPLATES
                    .render("pdf", &template_context)
                    .expect("Failed to render PDF template."),
            )
        } else if page_context.media_type == "text/html" {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url", &page_context.raw_url);

            Some(
                TEMPLATES
                    .render("page", &template_context)
                    .expect("Failed to render page template."),
            )
        } else {
            return None;
        };

        let mut template_context = tera::Context::new();
        template_context.insert("title", &page_context.title);
        template_context.insert("base_domain", &page_context.base_domain);
        template_context.insert("page_url", &page_context.page_url);
        template_context.insert("raw_url", &page_context.raw_url);
        template_context.insert("short_url", &page_context.short_url);
        template_context.insert("embed", &fragment);

        Some(
            TEMPLATES
                .render("base", &template_context)
                .map(FilePage)
                .expect("Failed to render template."),
        )
    }
}

pub fn render_css(base_domain: &str) -> String {
    let mut template_context = tera::Context::new();
    template_context.insert("base_domain", base_domain);

    TEMPLATES
        .render("style", &template_context)
        .expect("Failed to render style template.")
}

pub fn render_export_readme(exported_at: DateTime<Utc>) -> String {
    let mut template_context = tera::Context::new();
    template_context.insert(
        "timestamp",
        &exported_at.format("%-d %b %Y %H:%M:%S UTC").to_string(),
    );

    TEMPLATES
        .render("export-readme", &template_context)
        .expect("Failed to render export README template.")
}

// Optional fields are omitted rather than serialized as `null`, because Tera considers `null`
// values to be defined.

#[derive(Debug, Serialize)]
struct ReadmeItem<'a> {
    title: &'a str,
    summary: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    from_year: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_year: Option<u32>,
    people: Vec<ReadmeName<'a>>,
    identities: Vec<ReadmeName<'a>>,
    collections: Vec<ReadmeCollection<'a>>,
    links: Vec<ReadmeLink<'a>>,
    files: Vec<ReadmeFile<'a>>,
}

#[derive(Debug, Serialize)]
struct ReadmeName<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct ReadmeCollection<'a> {
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct ReadmeLink<'a> {
    title: &'a str,
    url: &'a str,
}

#[derive(Debug, Serialize)]
struct ReadmeFile<'a> {
    title: &'a str,
    filename: &'a str,
}

impl<'a> ReadmeItem<'a> {
    fn from_model(item: &'a models::Item) -> Self {
        Self {
            title: &item.title.value,
            summary: &item.summary.value,
            description: item.description.as_ref().map(|d| d.value.as_str()),
            from_year: item.from_year,
            to_year: item.to_year,
            people: item
                .people
                .iter()
                .map(|person| ReadmeName { name: &person.name })
                .collect(),
            identities: item
                .identities
                .iter()
                .map(|identity| ReadmeName {
                    name: &identity.name.value,
                })
                .collect(),
            collections: item
                .collections
                .iter()
                .map(|collection| ReadmeCollection {
                    title: &collection.title.value,
                    description: collection.description.as_ref().map(|d| d.value.as_str()),
                })
                .collect(),
            links: item
                .links
                .iter()
                .map(|link| ReadmeLink {
                    title: &link.title.value,
                    url: link.url.as_str(),
                })
                .collect(),
            files: item
                .files
                .iter()
                .map(|file| ReadmeFile {
                    title: &file.title.value,
                    filename: file.filename.as_ref(),
                })
                .collect(),
        }
    }
}

pub fn render_item_readme(item: &models::Item) -> String {
    let mut template_context = tera::Context::new();
    template_context.insert("item", &ReadmeItem::from_model(item));

    let readme = TEMPLATES
        .render("item-readme", &template_context)
        .expect("Failed to render item README template.");

    wrap_markdown(&readme, README_WIDTH)
}

const README_WIDTH: usize = 80;

/// Hard-wrap each line of a Markdown document, indenting continuation lines of list items.
///
/// Headings are left as-is because they can't span multiple lines, and words longer than the
/// width (like URLs) are never broken.
fn wrap_markdown(text: &str, width: usize) -> String {
    let mut wrapped = String::with_capacity(text.len());

    for line in text.split_inclusive('\n') {
        let (line, newline) = match line.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line, ""),
        };

        if line.chars().count() <= width || line.trim_start().starts_with('#') {
            wrapped.push_str(line);
            wrapped.push_str(newline);
            continue;
        }

        let prefix_len = list_prefix_len(line);
        let indent = " ".repeat(prefix_len);
        let mut current = String::from(&line[..prefix_len]);
        let mut current_len = prefix_len;
        let mut has_word = false;

        for word in line[prefix_len..].split_whitespace() {
            let word_len = word.chars().count();

            if has_word && current_len + 1 + word_len > width {
                wrapped.push_str(&current);
                wrapped.push('\n');
                current.clone_from(&indent);
                current_len = prefix_len;
                has_word = false;
            }

            if has_word {
                current.push(' ');
                current_len += 1;
            }

            current.push_str(word);
            current_len += word_len;
            has_word = true;
        }

        wrapped.push_str(&current);
        wrapped.push_str(newline);
    }

    wrapped
}

/// The length of a line's leading indentation and list marker (e.g. `- ` or `1. `), if any.
fn list_prefix_len(line: &str) -> usize {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent..];

    let marker = if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
        2
    } else {
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();

        if digits > 0 && rest[digits..].starts_with(". ") {
            digits + 2
        } else {
            0
        }
    };

    indent + marker
}

#[cfg(test)]
mod tests {
    use super::wrap_markdown;

    #[test]
    fn wraps_paragraphs() {
        let text = "one two three four five six\n";
        assert_eq!(wrap_markdown(text, 10), "one two\nthree four\nfive six\n");
    }

    #[test]
    fn indents_list_items() {
        let text = "- one two three four\n12. one two three\n";
        assert_eq!(
            wrap_markdown(text, 10),
            "- one two\n  three\n  four\n12. one\n    two\n    three\n"
        );
    }

    #[test]
    fn leaves_headings_and_long_words() {
        let text = "# a heading that is long\nhttps://example.com/long a\n";
        assert_eq!(
            wrap_markdown(text, 10),
            "# a heading that is long\nhttps://example.com/long\na\n"
        );
    }
}
