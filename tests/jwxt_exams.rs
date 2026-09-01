mod common;

use njupt::jwxt::Term;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_exams() {
    let jwxt = common::login().await;
    let exams = jwxt
        .student_exams(Some(2025), Some(Term::Second))
        .await
        .expect("student_exams");

    assert!(exams.as_json().get("items").is_some());
    eprintln!("exams: {} 场", exams.items.len());
}
