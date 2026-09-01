mod common;

use njupt::{login_card, login_card_via_webvpn, FetchMode};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_card_balance() {
    let card = if std::env::var("NJUPT_USE_WEBVPN").as_deref() == Ok("1") {
        login_card_via_webvpn(&common::creds())
            .await
            .expect("login_card_via_webvpn")
    } else {
        login_card(&common::creds()).await.expect("login_card")
    };

    let first = card
        .balance(FetchMode::CacheFirst)
        .await
        .expect("balance");
    assert!(!first.from_cache);
    assert!(first.data.amount >= 0.0);
    assert!(first.data.display.contains('元') || first.data.amount == 0.0);
    assert!(first.data.as_json().get("result").is_some());

    let cached = card
        .balance(FetchMode::CacheFirst)
        .await
        .expect("cached");
    assert!(cached.from_cache);
    assert_eq!(cached.data.amount, first.data.amount);

    let refreshed = card
        .balance(FetchMode::NetworkOnly)
        .await
        .expect("network");
    assert!(!refreshed.from_cache);

    eprintln!(
        "balance: {} ({}) from_cache={}",
        first.data.display, first.data.amount, cached.from_cache
    );
}
