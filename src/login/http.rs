use reqwest::{Client, redirect::Policy};
use std::time::Duration;

use crate::utils::Result;

// 创建一个默认的 HTTP 客户端 最多支持20次重定向 超时时间30秒 接受无效的证书
pub fn default_http_client() -> Result<Client> {
    Ok(Client::builder()
        .cookie_store(true)
        .redirect(Policy::limited(20))
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .user_agent(concat!("njupt-rs/", env!("CARGO_PKG_VERSION")))
        .build()?)
}
