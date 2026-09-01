pub mod card;
pub mod jwxt;
pub mod library;
pub mod login;
pub mod utils;

pub use card::{login_card, login_card_via_webvpn, Card, CardBalance};
pub use jwxt::exams::{Exam, ExamList};
pub use jwxt::grade_details::{GradeDetail, GradeDetailList};
pub use jwxt::grades::{Grade, GradeList};
pub use jwxt::profile::StudentProfile;
pub use jwxt::schedule::{Course, PracticeCourse, Schedule, StudentInfo};
pub use jwxt::selected::{SelectedCourse, SelectedCourseList};
pub use jwxt::{CacheKey, CacheKind, Cached, FetchMode, Jwxt, QueryResult, Term};
pub use login::{
    Credentials, DirectTransport, SsoClient, WebVpn, encode_host, login_jwxt,
    login_jwxt_via_webvpn, login_webvpn,
};
pub use utils::{Error, Result};
