use std::{fmt, sync::LazyLock};

use super::{config, resolver::MediaLocation};

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

    tera
});

#[derive(Debug, Clone)]
pub struct FilePageContext {
    pub title: String,
    pub base_domain: String,
    pub page_url: String,
    pub raw_url_path: String,
    pub short_url: String,
    pub media_type: String,
}

impl FilePageContext {
    pub fn from_location(location: &MediaLocation) -> Result<Self, anyhow::Error> {
        Ok(Self {
            title: location.filename.to_string(),
            base_domain: config::base_domain()?,
            page_url: location.page_url.to_string(),
            raw_url_path: location.raw_url.path().to_string(),
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
            template_context.insert("raw_url_path", &page_context.raw_url_path);

            Some(
                TEMPLATES
                    .render("image", &template_context)
                    .expect("Failed to render image template."),
            )
        } else if page_context.media_type.starts_with("video/") {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url_path", &page_context.raw_url_path);
            template_context.insert("media_type", &page_context.media_type);

            Some(
                TEMPLATES
                    .render("video", &template_context)
                    .expect("Failed to render video template."),
            )
        } else if page_context.media_type == "application/pdf" {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url_path", &page_context.raw_url_path);

            Some(
                TEMPLATES
                    .render("pdf", &template_context)
                    .expect("Failed to render PDF template."),
            )
        } else if page_context.media_type == "text/html" {
            let mut template_context = tera::Context::new();
            template_context.insert("raw_url_path", &page_context.raw_url_path);

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
        template_context.insert("raw_url_path", &page_context.raw_url_path);
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
