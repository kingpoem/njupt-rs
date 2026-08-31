mod credentials;
mod http;
mod transport;

pub mod sso;
pub mod webvpn;

pub use credentials::Credentials;
pub use http::default_http_client;
pub use sso::{LoginOutcome, SsoClient};
pub use transport::{DirectTransport, Transport};
pub use webvpn::{ENLINK_CAS_CALLBACK, OpenServiceResponse, WebVpn};

use crate::utils::Result;

pub async fn login_sso_for_service(
    creds: &Credentials,
    service: &str,
) -> Result<reqwest::Response> {
    let http = default_http_client()?;
    SsoClient::new(http)?.login_for_service(creds, service).await
}

pub async fn login_webvpn(creds: &Credentials) -> Result<WebVpn> {
    WebVpn::login_with_sso(creds).await
}
