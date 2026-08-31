use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::login::credentials::Credentials;
use crate::login::sso::crypto::{IAM_CHECK_KEY, iam_encrypt};
use crate::utils::{Error, Result};

pub const DEFAULT_IAM_BASE: &str = "http://i.njupt.edu.cn";
pub const DEFAULT_CAS_LOGIN: &str = "http://i.njupt.edu.cn/cas/login";

#[derive(Debug, Deserialize)]
struct LoginApiResponse {
    code: Option<i64>,
    message: Option<String>,
    result: Option<LoginResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResult {
    token: Option<String>,
    user_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct LoginOutcome {
    pub token: String,
    pub user_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct SsoClient {
    http: Client,
    iam_base: Url,
}

impl SsoClient {
    pub fn new(http: Client) -> Result<Self> {
        Ok(Self {
            http,
            iam_base: Url::parse(DEFAULT_IAM_BASE)?,
        })
    }

    pub fn with_iam_base(http: Client, iam_base: Url) -> Self {
        Self { http, iam_base }
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub async fn begin_cas(&self, service: &str) -> Result<String> {
        let mut cas = Url::parse(DEFAULT_CAS_LOGIN)?;
        cas.query_pairs_mut().append_pair("service", service);

        let response = self.http.get(cas).send().await?.error_for_status()?;
        response
            .url()
            .query_pairs()
            .find(|(k, _)| k == "service")
            .map(|(_, v)| v.into_owned())
            .ok_or(Error::MissingField("service"))
    }

    pub async fn login_password(&self, creds: &Credentials) -> Result<LoginOutcome> {
        let body = json!({
            "checkKey": IAM_CHECK_KEY,
            "username": iam_encrypt(&creds.username, IAM_CHECK_KEY)?,
            "password": iam_encrypt(&creds.password, IAM_CHECK_KEY)?,
            "mode": "none",
        });

        let payload: LoginApiResponse = self
            .http
            .post(self.iam_base.join("/ssoLogin/login")?)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let code = payload.code.unwrap_or(-1);
        if code != 200 {
            return Err(Error::Login(
                payload
                    .message
                    .unwrap_or_else(|| format!("sso login failed, code={code}")),
            ));
        }

        let token = payload
            .result
            .as_ref()
            .and_then(|r| r.token.clone())
            .ok_or(Error::MissingField("result.token"))?;

        Ok(LoginOutcome {
            token,
            user_info: payload.result.and_then(|r| r.user_info),
        })
    }

    pub async fn finish_cas(&self, session_id: &str) -> Result<reqwest::Response> {
        let mut url = self.iam_base.join("/ssoLogin/index")?;
        url.query_pairs_mut().append_pair("sessionId", session_id);
        Ok(self.http.get(url).send().await?.error_for_status()?)
    }

    /// 已有 TGC 时，经 CAS 为 `service` 落票（校外 service 应为 WebVPN 包装后的教务回调）。
    pub async fn goto_service(&self, service: &str) -> Result<reqwest::Response> {
        let mut cas = Url::parse(DEFAULT_CAS_LOGIN)?;
        cas.query_pairs_mut().append_pair("service", service);
        Ok(self.http.get(cas).send().await?.error_for_status()?)
    }

    pub async fn login_for_service(
        &self,
        creds: &Credentials,
        service: &str,
    ) -> Result<reqwest::Response> {
        let session_id = self.begin_cas(service).await?;
        self.login_password(creds).await?;
        self.finish_cas(&session_id).await
    }
}
