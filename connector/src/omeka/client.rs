use super::property::{Property, ResourceFilter, ResourceType};
use crate::config;

#[derive(Debug)]
pub struct FindQuery {
    properties: Vec<(Property, String)>,
    has_properties: Vec<Property>,
    filters: Vec<ResourceFilter>,
}

impl FindQuery {
    pub fn new() -> Self {
        FindQuery {
            properties: Vec::new(),
            has_properties: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn property<T: AsRef<str>>(&mut self, property: Property, value: T) -> &mut Self {
        self.properties.push((property, value.as_ref().to_string()));
        self
    }

    pub fn has_property(&mut self, property: Property) -> &mut Self {
        self.has_properties.push(property);
        self
    }

    pub fn filter(&mut self, filter: ResourceFilter) -> &mut Self {
        self.filters.push(filter);
        self
    }
}

/// A pointer to the next page of a paginated API response.
#[derive(Debug)]
pub struct NextPage {
    url: reqwest::Url,
}

impl NextPage {
    fn from_headers(headers: &reqwest::header::HeaderMap) -> anyhow::Result<Option<Self>> {
        let next_url = headers
            .get("Link")
            .and_then(|value| value.to_str().ok())
            .map(parse_link_header::parse)
            .transpose()?
            .and_then(|header| {
                header
                    .get(&Some(String::from("next")))
                    .map(|next| next.raw_uri.clone())
            })
            .map(|url| reqwest::Url::parse(&url))
            .transpose()?;

        Ok(next_url.map(|url| NextPage { url }))
    }
}

#[derive(Debug)]
pub struct Client {
    pub base_url: reqwest::Url,
    pub client: reqwest::Client,
}

impl Client {
    fn new(client: reqwest::Client, base_url: reqwest::Url) -> Self {
        Client { base_url, client }
    }

    pub fn from_config(client: reqwest::Client) -> anyhow::Result<Self> {
        Ok(Client::new(client, config::omeka_url()?))
    }

    pub async fn find(
        &self,
        resource: ResourceType,
        query: &FindQuery,
    ) -> anyhow::Result<(reqwest::Response, Option<NextPage>)> {
        let mut url = self.base_url.join("/api")?;

        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Failed to build Omeka API endpoint path."))?
            .push(resource.as_str());

        {
            let mut query_pairs_mut = url.query_pairs_mut();

            for filter in &query.filters {
                let (key, value) = filter.as_query_param();
                query_pairs_mut.append_pair(&key, &value);
            }

            let mut counter = 0;

            for property in &query.has_properties {
                query_pairs_mut.extend_pairs(&[
                    (
                        format!("property[{}][property]", counter),
                        property.as_str(),
                    ),
                    (format!("property[{}][type]", counter), "ex"),
                ]);

                counter += 1;
            }

            for (property, value) in &query.properties {
                query_pairs_mut.extend_pairs(&[
                    (
                        format!("property[{}][property]", counter),
                        property.as_str(),
                    ),
                    (format!("property[{}][type]", counter), "eq"),
                    (format!("property[{}][text]", counter), value),
                ]);

                counter += 1;
            }

            query_pairs_mut.finish();
        }

        let response = self.client.get(url.clone()).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Omeka request failed with {}: {}",
                response.status(),
                url.as_str()
            );
        }

        let next_page = NextPage::from_headers(response.headers())?;

        Ok((response, next_page))
    }

    pub async fn next_page(
        &self,
        next_page: NextPage,
    ) -> anyhow::Result<(reqwest::Response, Option<NextPage>)> {
        let response = self.client.get(next_page.url.clone()).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Omeka request failed with {}: {}",
                response.status(),
                next_page.url.as_str()
            );
        }

        let next_page = NextPage::from_headers(response.headers())?;

        Ok((response, next_page))
    }
}
