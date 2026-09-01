mod common;

use njupt::jwxt::{FetchMode, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_selected_courses() {
    let jwxt = common::login().await;
    let selected = jwxt
        .selected_courses(Some(2025), Some(Term::Second), FetchMode::CacheFirst)
        .await
        .expect("selected_courses");

    assert!(!selected.data.items.is_empty());
    assert!(!selected.data.items[0].code.is_empty());
    eprintln!(
        "selected: {} 门, from_cache={}",
        selected.data.items.len(),
        selected.from_cache
    );
}
