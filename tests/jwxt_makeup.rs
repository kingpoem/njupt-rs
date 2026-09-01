mod common;

use njupt::jwxt::{FetchMode, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_makeup_and_deferred_exams() {
    let jwxt = common::login().await;

    let makeup = jwxt
        .makeup_exams(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("makeup_exams");
    assert!(!makeup.from_cache);
    assert!(makeup.data.as_json().get("items").is_some());

    let makeup_cached = jwxt
        .makeup_exams(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("makeup_cached");
    assert!(makeup_cached.from_cache);
    assert_eq!(makeup_cached.data.items.len(), makeup.data.items.len());

    let deferred = jwxt
        .deferred_exams(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("deferred_exams");
    assert!(!deferred.from_cache);
    assert!(deferred.data.as_json().get("items").is_some());

    let deferred_cached = jwxt
        .deferred_exams(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("deferred_cached");
    assert!(deferred_cached.from_cache);
    assert_eq!(deferred_cached.data.items.len(), deferred.data.items.len());

    eprintln!(
        "makeup: {} 场, deferred: {} 场, cache ok",
        makeup.data.items.len(),
        deferred.data.items.len()
    );
}
