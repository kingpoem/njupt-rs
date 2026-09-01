pub mod balance;

pub use balance::CardBalance;

use std::sync::{Arc, Mutex};

use reqwest::Client;
use serde_json::Value;

use crate::login::sso::SsoClient;
use crate::login::webvpn::{WebVpn, ENLINK_CAS_CALLBACK};
use crate::login::{default_http_client, Credentials};
use crate::utils::{Error, Result};

pub const PORTAL_BASE: &str = "https://i.njupt.edu.cn";
pub const PORTAL_HOME: &str = "https://i.njupt.edu.cn/portal";
pub const YKT_BALANCE_PATH: &str = "/portal/api/getYktYE";

#[derive(Debug, Clone)]
pub struct Card {
    http: Client,
    balance_cache: Arc<Mutex<Option<Value>>>,
}

impl Card {
    pub fn with_http(http: Client) -> Self {
        Self {
            http,
            balance_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub fn clear_cache(&self) {
        if let Ok(mut guard) = self.balance_cache.lock() {
            *guard = None;
        }
    }

    pub(crate) async fn ensure_portal(&self) -> Result<()> {
        let sso = SsoClient::new(self.http.clone())?;
        let response = sso.goto_service(PORTAL_HOME).await?;
        let landed = response.url().as_str();
        if landed.contains("cas/login") || landed.contains("user-login") {
            return Err(Error::Login(format!(
                "portal CAS ticket not accepted, landed on {landed}"
            )));
        }
        let _ = self
            .http
            .get(PORTAL_HOME)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub(crate) fn balance_cache_get(&self) -> Option<Value> {
        self.balance_cache
            .lock()
            .ok()
            .and_then(|g| g.as_ref().cloned())
    }

    pub(crate) fn balance_cache_store(&self, value: Value) {
        if let Ok(mut guard) = self.balance_cache.lock() {
            *guard = Some(value);
        }
    }
}

/// 校内：SSO 登录后访问智慧校园门户查卡余额。
pub async fn login_card(creds: &Credentials) -> Result<Card> {
    let http = default_http_client()?;
    SsoClient::new(http.clone())?
        .login_for_service(creds, PORTAL_HOME)
        .await?;
    Ok(Card::with_http(http))
}

/// 校外：经 Enlink WebVPN 完成 SSO，再直连门户 API。
pub async fn login_card_via_webvpn(creds: &Credentials) -> Result<Card> {
    let http = default_http_client()?;
    SsoClient::new(http.clone())?
        .login_for_service(creds, ENLINK_CAS_CALLBACK)
        .await?;
    Ok(Card::with_http(http))
}

impl WebVpn {
    pub fn into_card(self) -> Card {
        Card::with_http(self.http().clone())
    }
}
