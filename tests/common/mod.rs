use njupt::{Credentials, Jwxt, login_jwxt, login_jwxt_via_webvpn};

use std::path::Path;

pub fn creds() -> Credentials {
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&env_path);

    let username = std::env::var("NJUPT_USERNAME").unwrap_or_default();
    let password = std::env::var("NJUPT_PASSWORD").unwrap_or_default();
    assert!(
        !username.is_empty() && !password.is_empty(),
        "请在项目根目录 .env 中填写 NJUPT_USERNAME / NJUPT_PASSWORD（可参考 .env.example）"
    );
    Credentials::new(username, password)
}

/// 校内优先 `login_jwxt`；环境变量 `NJUPT_USE_WEBVPN=1` 时走 WebVPN。
pub async fn login() -> Jwxt {
    let creds = creds();
    if std::env::var("NJUPT_USE_WEBVPN").as_deref() == Ok("1") {
        login_jwxt_via_webvpn(&creds)
            .await
            .expect("login_jwxt_via_webvpn")
    } else {
        login_jwxt(&creds).await.expect("login_jwxt")
    }
}

#[allow(dead_code)]
pub fn item_count(v: &serde_json::Value) -> usize {
    v.get("items")
        .and_then(|i| i.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn dump_json(label: &str, v: &serde_json::Value) {
    eprintln!("--- {label} ---");
    eprintln!("{}", serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()));
}
