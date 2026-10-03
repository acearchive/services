use secrecy::{ExposeSecret, SecretString};

use super::property::Property;
use crate::config;

#[derive(Debug)]
pub struct ApiKeyId(SecretString);

impl From<String> for ApiKeyId {
    fn from(value: String) -> Self {
        ApiKeyId(SecretString::from(value))
    }
}

impl ExposeSecret<str> for ApiKeyId {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

#[derive(Debug)]
pub struct ApiKeyCred(SecretString);

impl From<String> for ApiKeyCred {
    fn from(value: String) -> Self {
        ApiKeyCred(SecretString::from(value))
    }
}

impl ExposeSecret<str> for ApiKeyCred {
    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

#[derive(Debug)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub cred: ApiKeyCred,
}

#[derive(Debug)]
pub struct Client {
    pub base_url: String,
    pub key: ApiKey,
    pub client: reqwest::Client,
}

impl Client {
    fn new(base_url: String, key: ApiKey) -> Self {
        Client {
            base_url,
            key,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config() -> anyhow::Result<Self> {
        let base_url = config::omeka_url()?;

        let key = ApiKey {
            id: config::omeka_key_id()?,
            cred: config::omeka_key_cred()?,
        };

        Ok(Client::new(base_url, key))
    }

    fn base_endpoint(&self) -> anyhow::Result<reqwest::Url> {
        let mut url = reqwest::Url::parse(&self.base_url)?;

        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to build Omeka API endpoint path."))?
            .push("api");

        url.query_pairs_mut()
            .extend_pairs(&[
                ("key_identity", self.key.id.expose_secret()),
                ("key_credential", self.key.cred.expose_secret()),
            ])
            .finish();

        Ok(url)
    }

    pub async fn find_by_property(
        &self,
        property: Property,
        value: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let mut url = self.base_endpoint()?;

        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to build Omeka API endpoint path."))?
            .push("items");

        url.query_pairs_mut()
            .extend_pairs(&[
                ("property[0][property]", property.as_str()),
                ("property[0][type]", "eq"),
                ("property[0][text]", value),
            ])
            .finish();

        let response = self.client.get(url).send().await?;

        Ok(response)
    }
}
