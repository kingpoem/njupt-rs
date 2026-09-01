use reqwest::Client;
use url::Url;

use super::crypto::encode_host;
use crate::jwxt::Jwxt;
use crate::login::credentials::Credentials;
use crate::login::http::default_http_client;
use crate::login::sso::SsoClient;
use crate::utils::{Error, Result};

pub const DEFAULT_WEBVPN_BASE: &str = "https://vpn.njupt.edu.cn:8443";
pub const ENLINK_CAS_CALLBACK: &str =
    "https://vpn.njupt.edu.cn:8443/enlink/api/client/callback/cas";

#[derive(Debug, Clone)]
pub struct WebVpn {
    http: Client,
    base: Url,
}

impl WebVpn {
    pub fn base(&self) -> &Url {
        &self.base
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub async fn login_with_sso(creds: &Credentials) -> Result<Self> {
        let http = default_http_client()?;
        let sso = SsoClient::new(http.clone())?;
        let response = sso.login_for_service(creds, ENLINK_CAS_CALLBACK).await?;
        let landed = response.url().as_str();
        if !(landed.contains("vpn.njupt.edu.cn") || landed.contains("/enlink")) {
            return Err(Error::Unexpected(format!(
                "expected Enlink landing URL after SSO, got {landed}"
            )));
        }

        Ok(Self {
            http,
            base: Url::parse(DEFAULT_WEBVPN_BASE)?,
        })
    }

    pub fn into_jwxt(self) -> Jwxt {
        Jwxt::with_http(self.http)
    }

    pub fn wrap_url(&self, target: &Url) -> Result<Url> {
        let host = target
            .host_str()
            .ok_or(Error::MissingField("url.host"))?;
        let host_hash = encode_host(host)?;
        let path = target.path().trim_start_matches('/');
        let query = target
            .query()
            .map(|q| format!("?{q}"))
            .unwrap_or_default();
        let wrapped = format!(
            "{}/{}/webvpn{host_hash}/{path}{query}",
            self.base.as_str().trim_end_matches('/'),
            target.scheme(),
        );
        Ok(Url::parse(&wrapped)?)
    }

    pub async fn get(&self, url: &Url) -> Result<reqwest::Response> {
        Ok(self.http.get(url.clone()).send().await?)
    }
}
