// 测试 Enlink WebVPN 登录与应用列表；需网络，账密见项目根目录 .env
mod common;

use njupt::login_webvpn;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn webvpn_login_with_sso() {
    let creds = common::creds();
    let vpn = login_webvpn(&creds).await.expect("webvpn login");
    assert!(vpn.base().as_str().contains("vpn.njupt.edu.cn"));
}

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn webvpn_list_service_tree() {
    let creds = common::creds();
    let vpn = login_webvpn(&creds).await.expect("webvpn login");
    let tree = vpn.list_service_tree().await.expect("service tree");
    assert!(!tree.is_null());
}
