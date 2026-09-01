mod common;

use njupt::jwxt::{FetchMode, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_grades() {
    let jwxt = common::login().await;

    let all = jwxt
        .student_grades(None, None, FetchMode::CacheFirst)
        .await
        .expect("all");
    assert!(!all.from_cache);
    assert!(!all.data.items.is_empty());
    assert!(all.data.as_json().get("items").is_some());
    assert!(!all.data.items[0].name.is_empty());
    assert!(all.data.items[0].credit > 0.0);
    assert!(all.data.items[0].grade_point.is_some());

    let term = jwxt
        .student_grades(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("term");
    assert!(term.data.items.iter().all(|g| g.year == "2025"));

    let cached = jwxt
        .student_grades(None, None, FetchMode::CacheFirst)
        .await
        .expect("cached");
    assert!(cached.from_cache);
    assert_eq!(cached.data.items.len(), all.data.items.len());

    eprintln!(
        "all: {} 门, sample={}, from_cache={}",
        all.data.items.len(),
        all.data.items[0].name,
        cached.from_cache
    );
}
