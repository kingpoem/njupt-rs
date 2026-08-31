// 核心联调：SSO → WebVPN → 教务课表；账密见项目根目录 .env
mod common;

use njupt::jwxt::{Jwxt, Term};
use njupt::login_webvpn;

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn webvpn_fetch_student_schedule() {
    let creds = common::creds();
    let vpn = login_webvpn(&creds).await.expect("login_webvpn");
    let schedule = Jwxt::new(vpn)
        .student_schedule(2025, Term::Second)
        .await
        .expect("student_schedule");

    assert_eq!(schedule.student.student_id, creds.username);
    assert!(!schedule.courses.is_empty() || !schedule.practices.is_empty());
}
