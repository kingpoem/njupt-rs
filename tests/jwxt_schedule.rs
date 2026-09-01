mod common;

use njupt::jwxt::{FetchMode, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_schedule() {
    let creds = common::creds();
    let jwxt = common::login().await;
    let cached = jwxt
        .student_schedule(2025, Term::Second, FetchMode::CacheFirst)
        .await
        .expect("student_schedule");
    let schedule = &cached.data;

    assert!(!cached.from_cache);
    assert_eq!(schedule.student.student_id, creds.username);
    assert!(!schedule.courses.is_empty());
    assert!(!schedule.courses[0].name.is_empty());
    let raw_course = schedule.as_json()["kbList"][0]["kcmc"]
        .as_str()
        .unwrap_or("");
    assert_eq!(schedule.courses[0].name, raw_course);

    let again = jwxt
        .student_schedule(2025, Term::Second, FetchMode::CacheFirst)
        .await
        .expect("cached");
    assert!(again.from_cache);

    eprintln!(
        "courses: {}, practices: {}",
        schedule.courses.len(),
        schedule.practices.len()
    );
}
