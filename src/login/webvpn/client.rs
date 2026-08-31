use reqwest::Client;
use serde::Deserialize;
use url::Url;

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

    pub fn wrap_known_host(&self, target: &Url, host_hash: &str) -> Result<Url> {
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

    pub async fn list_service_tree(&self) -> Result<serde_json::Value> {
        let url = self
            .base
            .join("/enlink/api/client/service/group/treeWithService")?;
        Ok(self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn open_service(&self, service_id: &str) -> Result<OpenServiceResponse> {
        let url = self
            .base
            .join(&format!("/enlink/api/client/service/v2/open/{service_id}"))?;
        Ok(self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn get(&self, url: &Url) -> Result<reqwest::Response> {
        Ok(self.http.get(url.clone()).send().await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenServiceResponse {
    pub code: Option<String>,
    pub messages: Option<String>,
    pub data: Option<serde_json::Value>,
}
