use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::login::credentials::Credentials;
use crate::login::http::default_http_client;
use crate::login::sso::SsoClient;
use crate::login::transport::Transport;
use crate::utils::{Error, Result};

pub const DEFAULT_WEBVPN_BASE: &str = "https://vpn.njupt.edu.cn:8443";
pub const ENLINK_CAS_CALLBACK: &str =
    "https://vpn.njupt.edu.cn:8443/enlink/api/client/callback/cas";

/// Enlink 为 `jwglxt.njupt.edu.cn` 生成的稳定 host hash（`/http/webvpn{hash}/...`）。
pub const JWGLXT_HOST_HASH: &str =
    "35f994eca13c99939dc072124de9e7e8d058298921ede2bb096b0d45fab5cbe1";

pub const JWGLXT_HOST: &str = "jwglxt.njupt.edu.cn";

#[derive(Debug, Clone)]
pub struct WebVpn {
    http: Client,
    base: Url,
    host_hashes: HashMap<String, String>,
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

        let mut host_hashes = HashMap::new();
        host_hashes.insert(JWGLXT_HOST.to_string(), JWGLXT_HOST_HASH.to_string());

        Ok(Self {
            http,
            base: Url::parse(DEFAULT_WEBVPN_BASE)?,
            host_hashes,
        })
    }

    pub fn register_host_hash(&mut self, host: impl Into<String>, hash: impl Into<String>) {
        self.host_hashes.insert(host.into(), hash.into());
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
        let payload = serde_json::json!({
            "nameLike": "",
            "serviceNameLike": "",
            "userId": null,
        });
        let body: serde_json::Value = self
            .http
            .post(url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(body)
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

impl Transport for WebVpn {
    fn map_url(&self, target: &Url) -> Result<Url> {
        let host = target
            .host_str()
            .ok_or(Error::MissingField("url.host"))?;
        let hash = self
            .host_hashes
            .get(host)
            .ok_or_else(|| Error::Unexpected(format!("no webvpn host hash registered for {host}")))?;
        self.wrap_known_host(target, hash)
    }

    fn http(&self) -> &Client {
        &self.http
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenServiceResponse {
    pub code: Option<String>,
    pub messages: Option<String>,
    pub data: Option<serde_json::Value>,
}
