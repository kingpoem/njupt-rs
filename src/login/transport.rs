use url::Url;

use crate::utils::Result;

pub trait Transport: Send + Sync {
    fn map_url(&self, target: &Url) -> Result<Url>;
    fn http(&self) -> &reqwest::Client;
}

#[derive(Debug, Clone)]
pub struct DirectTransport {
    http: reqwest::Client,
}

impl DirectTransport {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }
}

impl Transport for DirectTransport {
    fn map_url(&self, target: &Url) -> Result<Url> {
        Ok(target.clone())
    }

    fn http(&self) -> &reqwest::Client {
        &self.http
    }
}
