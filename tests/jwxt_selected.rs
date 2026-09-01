mod common;

use njupt::jwxt::Term;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_selected_courses() {
    let jwxt = common::login().await;
    let selected = jwxt
        .selected_courses(Some(2025), Some(Term::Second))
        .await
        .expect("selected_courses");

    assert!(!selected.items.is_empty());
    assert!(!selected.items[0].code.is_empty());
    eprintln!("selected: {} 门", selected.items.len());
}
