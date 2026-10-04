use secrecy::{ExposeSecret, SecretString};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
