use super::{config, resolver::MediaLocator};

pub fn format(format: MediaLocator) -> anyhow::Result<reqwest::Url> {
    let mut url = config::files_url()?;

    {
        let mut path_segments = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to get path segments from files URL."))?;

        match format {
            MediaLocator::Long { slug, filename } => {
                path_segments.extend(["artifacts", slug.as_ref(), filename.as_ref()]);
            }
            MediaLocator::Short { id, filename } => {
                path_segments.extend(["a", id.as_ref(), filename.as_ref()]);
            }
            MediaLocator::Raw { id, filename } => {
                path_segments.extend(["r", id.as_ref(), filename.as_ref()]);
            }
        }
    }

    Ok(url)
}
