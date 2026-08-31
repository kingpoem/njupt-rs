// 测试校外用法示例（SSO→WebVPN）；联调账密见项目根目录 .env，占位学号密码仅测 API 形状
mod common;

use njupt::{Credentials, login_webvpn};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn example_offcampus_sso_webvpn() {
    let creds = common::creds();
    let vpn = login_webvpn(&creds).await.expect("login_webvpn");
    let apps = vpn.list_service_tree().await.expect("list_service_tree");
    assert!(!apps.is_null());
}

#[test]
fn example_offcampus_api_shape() {
    let _ = Credentials::new("Bxxxxxxxx", "your-password");
}
