use super::{config, omeka};
pub enum Format {
    Long {
        slug: omeka::AceSlug,
        filename: omeka::AceFilename,
    },
    Short {
        id: omeka::AceId,
        filename: omeka::AceFilename,
    },
    Raw {
        id: omeka::AceId,
        filename: omeka::AceFilename,
    },
}

pub fn get(format: Format) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url()?;

    {
        let mut path_segments = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to get path segments from files URL."))?;

        match format {
            Format::Long { slug, filename } => {
                path_segments.extend(["artifacts", slug.as_ref(), filename.as_ref()]);
            }
            Format::Short { id, filename } => {
                path_segments.extend(["a", id.as_ref(), filename.as_ref()]);
            }
            Format::Raw { id, filename } => {
                path_segments.extend(["r", id.as_ref(), filename.as_ref()]);
            }
        }
    }

    Ok(url)
}
