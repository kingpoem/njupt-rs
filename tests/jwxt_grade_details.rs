mod common;

use njupt::jwxt::{FetchMode, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_grade_details() {
    let jwxt = common::login().await;

    let term = jwxt
        .grade_details(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("grade_details");
    assert!(!term.from_cache);
    assert!(!term.data.items.is_empty());
    assert!(term.data.as_json().get("items").is_some());
    assert!(term.data.items.iter().all(|d| d.year == "2025"));

    let sample = &term.data.items[0];
    assert!(!sample.name.is_empty());
    assert!(!sample.component.is_empty());
    assert!(!sample.score.is_empty());

    let cached = jwxt
        .grade_details(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("cached");
    assert!(cached.from_cache);
    assert_eq!(cached.data.items.len(), term.data.items.len());

    let raw = jwxt
        .grade_details_json(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("grade_details_json");
    assert!(raw.from_cache);
    assert_eq!(
        common::item_count(&raw.data),
        term.data.items.len()
    );

    eprintln!(
        "grade_details: {} 条, sample={} / {} = {}, from_cache={}",
        term.data.items.len(),
        sample.name,
        sample.component,
        sample.score,
        cached.from_cache
    );
}
