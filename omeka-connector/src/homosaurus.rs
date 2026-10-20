use serde::Deserialize;

use crate::omeka::LiteralPropertyValue;

/// A JSON-LD property value which may be a single value or an array of values.
///
/// Compacted JSON-LD collapses single-element arrays into a single value.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        OneOrMany::Many(Vec::new())
    }
}

impl<T> OneOrMany<T> {
    fn as_slice(&self) -> &[T] {
        match self {
            OneOrMany::One(value) => std::slice::from_ref(value),
            OneOrMany::Many(values) => values.as_slice(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConceptResponse {
    #[serde(default, rename = "rdfs:comment")]
    comment: OneOrMany<LiteralPropertyValue<String>>,
}

/// Return whether this URL points to a term in the Homosaurus vocabulary.
pub fn is_homosaurus_url(url: &reqwest::Url) -> bool {
    url.host_str() == Some("homosaurus.org")
}

#[derive(Debug, Clone)]
pub struct Client {
    pub client: reqwest::Client,
}

impl Client {
    pub fn new(client: reqwest::Client) -> Self {
        Client { client }
    }

    /// Return the English definition of the Homosaurus term at this URL, if it has one.
    pub async fn english_definition(&self, url: &reqwest::Url) -> anyhow::Result<Option<String>> {
        // Requesting the term URL with an `Accept: application/ld+json` header returns JSON-LD
        // with the language tags stripped, so we use the `.jsonld` endpoint instead.
        let mut url = url.clone();
        url.set_path(&format!("{}.jsonld", url.path()));

        let response = self.client.get(url.clone()).send().await?;

        log::info!("GET {}", url.as_str());

        if !response.status().is_success() {
            anyhow::bail!(
                "Homosaurus request failed with {}: {}",
                response.status(),
                url.as_str()
            );
        }

        let concept = response.json::<ConceptResponse>().await?;

        Ok(concept
            .comment
            .as_slice()
            .iter()
            .find(|comment| comment.lang.as_deref() == Some("en"))
            .map(|comment| comment.value.clone()))
    }
}
