use secrecy::SecretString;

#[derive(Debug)]
pub struct ApiKeyIdentity(SecretString);

#[derive(Debug)]
pub struct ApiKeyCredential(SecretString);

#[derive(Debug)]
pub struct ApiKey {
    pub id: ApiKeyIdentity,
    pub cred: ApiKeyCredential,
}

#[derive(Debug)]
pub struct Client {
    pub base_url: String,
    pub key: ApiKey,
    pub client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: String, key: ApiKey) -> Self {
        Client {
            base_url,
            key,
            client: reqwest::Client::new(),
        }
    }

    fn base_endpoint(&self) -> anyhow::Result<&str> {
        let url = reqwest::Url::parse(&self.base_url);
        todo!()
    }
}
