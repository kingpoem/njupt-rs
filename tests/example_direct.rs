// 测试校内用法示例（SSO→Direct→Jwxt）；联调账密见项目根目录 .env，service URL 可按教务回调修改
mod common;

use njupt::jwxt::Jwxt;
use njupt::login::{DirectTransport, SsoClient, default_http_client};
use njupt::Credentials;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn example_oncampus_sso_direct_jwxt() {
    let creds = common::creds();
    let http = default_http_client().expect("http client");
    let sso = SsoClient::new(http.clone()).expect("sso client");
    let _ = sso
        .login_for_service(&creds, "http://jwglxt.njupt.edu.cn/sso/ddlogin")
        .await
        .expect("sso login for jwxt");
    let jwxt = Jwxt::new(DirectTransport::new(http));
    let _ = jwxt.transport();
}

#[test]
fn example_oncampus_api_shape() {
    let http = default_http_client().expect("http client");
    let creds = Credentials::new("Bxxxxxxxx", "your-password");
    let _ = SsoClient::new(http.clone()).expect("sso client");
    let jwxt = Jwxt::new(DirectTransport::new(http));
    let _ = (creds, jwxt.transport());
}
