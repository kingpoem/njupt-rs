mod common;

use njupt::jwxt::Term;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_grades() {
    let jwxt = common::login().await;

    let all = jwxt.student_grades(None, None).await.expect("all");
    assert!(!all.items.is_empty());
    assert!(all.as_json().get("items").is_some());
    assert!(!all.items[0].name.is_empty());
    assert!(all.items[0].credit > 0.0);
    assert!(all.items[0].grade_point.is_some());

    let term = jwxt
        .student_grades(Some(2025), Some(Term::Second))
        .await
        .expect("term");
    assert!(term.items.iter().all(|g| g.year == "2025"));

    eprintln!("all: {} 门, sample={}", all.items.len(), all.items[0].name);
}
