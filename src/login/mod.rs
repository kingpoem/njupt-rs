mod credentials;
mod http;
mod transport;

pub mod sso;
pub mod webvpn;

pub use credentials::Credentials;
pub use http::default_http_client;
pub use sso::{LoginOutcome, SsoClient};
pub use transport::{DirectTransport, Transport};
pub use webvpn::{WebVpn, encode_host};

use crate::jwxt::{Jwxt, JWGLXT_DDLOGIN_SERVICE};
use crate::utils::Result;

/// 校内：SSO 登录后直接访问教务。
pub async fn login_jwxt(creds: &Credentials) -> Result<Jwxt> {
    let http = default_http_client()?;
    SsoClient::new(http.clone())?
        .login_for_service(creds, JWGLXT_DDLOGIN_SERVICE)
        .await?;
    Ok(Jwxt::with_http(http))
}

/// 校外：经 Enlink WebVPN 完成 SSO，再直连教务 API。
pub async fn login_jwxt_via_webvpn(creds: &Credentials) -> Result<Jwxt> {
    Ok(WebVpn::login_with_sso(creds).await?.into_jwxt())
}

/// 登录 Enlink WebVPN 门户；用于 `wrap_url` 代理访问其他站点（如知网）。
pub async fn login_webvpn(creds: &Credentials) -> Result<WebVpn> {
    WebVpn::login_with_sso(creds).await
}
