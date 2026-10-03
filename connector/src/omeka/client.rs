use secrecy::ExposeSecret;

use super::creds::ApiKey;
use super::property::{Property, ResourceFilter, ResourceType};
use crate::config;

#[derive(Debug)]
pub struct FindQuery {
    properties: Vec<(Property, String)>,
    filters: Vec<ResourceFilter>,
}

impl FindQuery {
    pub fn new() -> Self {
        FindQuery {
            properties: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn property<T: AsRef<str>>(&mut self, property: Property, value: T) -> &mut Self {
        self.properties.push((property, value.as_ref().to_string()));
        self
    }

    pub fn filter(&mut self, filter: ResourceFilter) -> &mut Self {
        self.filters.push(filter);
        self
    }
}

#[derive(Debug)]
pub struct Client {
    pub base_url: reqwest::Url,
    pub key: ApiKey,
    pub client: reqwest::Client,
}

impl Client {
    fn new(base_url: reqwest::Url, key: ApiKey) -> Self {
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
        let mut url = self.base_url.clone();

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

    pub async fn find(
        &self,
        resource: ResourceType,
        query: &FindQuery,
    ) -> anyhow::Result<reqwest::Response> {
        let mut url = self.base_endpoint()?;

        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to build Omeka API endpoint path."))?
            .push(resource.as_str());

        {
            let mut query_pairs_mut = url.query_pairs_mut();

            for (property, value) in &query.properties {
                query_pairs_mut.extend_pairs(&[
                    ("property[0][property]", property.as_str()),
                    ("property[0][type]", "eq"),
                    ("property[0][text]", &value),
                ]);
            }

            for filter in &query.filters {
                let (key, value) = filter.as_query_param();
                query_pairs_mut.append_pair(&key, &value);
            }

            query_pairs_mut.finish();
        }

        let response = self.client.get(url).send().await?;

        Ok(response)
    }
}
