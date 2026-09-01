mod common;

use njupt::jwxt::Term;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_schedule() {
    let creds = common::creds();
    let jwxt = common::login().await;
    let schedule = jwxt
        .student_schedule(2025, Term::Second)
        .await
        .expect("student_schedule");

    assert_eq!(schedule.student.student_id, creds.username);
    assert!(!schedule.courses.is_empty());
    assert!(!schedule.courses[0].name.is_empty());
    let raw_course = schedule.as_json()["kbList"][0]["kcmc"]
        .as_str()
        .unwrap_or("");
    assert_eq!(schedule.courses[0].name, raw_course);

    eprintln!(
        "courses: {}, practices: {}",
        schedule.courses.len(),
        schedule.practices.len()
    );
}
