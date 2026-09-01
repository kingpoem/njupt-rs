mod common;

use njupt::jwxt::FetchMode;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn fetch_student_profile() {
    let creds = common::creds();
    let jwxt = common::login().await;
    let cached = jwxt
        .student_profile(FetchMode::CacheFirst)
        .await
        .expect("student_profile");
    let profile = &cached.data;

    assert!(!cached.from_cache);
    assert_eq!(profile.student_id, creds.username);
    assert_eq!(profile.name, profile.field("姓名").unwrap_or(""));
    assert!(profile.as_json().as_object().is_some_and(|m| m.len() > 10));

    let again = jwxt
        .student_profile(FetchMode::CacheFirst)
        .await
        .expect("cached");
    assert!(again.from_cache);

    eprintln!(
        "name={}, fields={}",
        profile.name,
        profile.as_json().as_object().map(|m| m.len()).unwrap_or(0)
    );
}
