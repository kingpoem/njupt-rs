use reqwest::{Client, redirect::Policy};

use crate::utils::Result;

pub fn default_http_client() -> Result<Client> {
    Ok(Client::builder()
        .cookie_store(true)
        .redirect(Policy::limited(20))
        .danger_accept_invalid_certs(true)
        .user_agent(concat!("njupt-rs/", env!("CARGO_PKG_VERSION")))
        .build()?)
}
