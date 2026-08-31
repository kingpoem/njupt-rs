// 测试 SSO/CAS 登录；联调需网络，账密见项目根目录 .env
mod common;

use njupt::login::{ENLINK_CAS_CALLBACK, SsoClient, default_http_client};

#[tokio::test]
#[ignore = "needs network access to i.njupt.edu.cn"]
async fn sso_begin_cas_returns_session_id() {
    let http = default_http_client().expect("http client");
    let sso = SsoClient::new(http).expect("sso client");
    let session_id = sso
        .begin_cas(ENLINK_CAS_CALLBACK)
        .await
        .expect("begin cas");
    assert!(!session_id.is_empty());
}

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn sso_login_for_enlink_callback() {
    let creds = common::creds();
    let http = default_http_client().expect("http client");
    let sso = SsoClient::new(http).expect("sso client");
    let response = sso
        .login_for_service(&creds, ENLINK_CAS_CALLBACK)
        .await
        .expect("sso login");
    let url = response.url().as_str();
    assert!(
        url.contains("vpn.njupt.edu.cn") || url.contains("/enlink") || url.contains("i.njupt.edu.cn"),
        "unexpected landing url: {url}"
    );
}
