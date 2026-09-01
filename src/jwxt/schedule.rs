use serde::Serialize;
use serde_json::Value;

use super::{
    CacheKey, Cached, FetchMode, Jwxt, SCHEDULE_GNMKDM, Term, f64_field, map_array, parse_json_value,
    str_field,
};
use crate::utils::Result;

#[derive(Debug, Clone, Serialize)]
pub struct StudentInfo {
    pub student_id: String,
    pub name: String,
    pub class_name: String,
    pub major: String,
    pub academic_year: String,
    pub year: String,
    pub term_code: String,
    pub term_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Course {
    pub name: String,
    pub code: String,
    pub teacher: String,
    pub room: String,
    pub weekday: String,
    pub weekday_name: String,
    pub sections: String,
    pub weeks: String,
    pub credit: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PracticeCourse {
    pub name: String,
    pub teacher: String,
    pub weeks: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Schedule {
    raw: Value,
    pub student: StudentInfo,
    pub courses: Vec<Course>,
    pub practices: Vec<PracticeCourse>,
}

impl StudentInfo {
    fn from_value(v: &Value) -> Self {
        Self {
            student_id: str_field(v, "XH"),
            name: str_field(v, "XM"),
            class_name: str_field(v, "BJMC"),
            major: str_field(v, "ZYMC"),
            academic_year: str_field(v, "XNMC"),
            year: str_field(v, "XNM"),
            term_code: str_field(v, "XQM"),
            term_name: str_field(v, "XQMMC"),
        }
    }
}

impl Course {
    fn from_value(v: &Value) -> Self {
        Self {
            name: str_field(v, "kcmc"),
            code: str_field(v, "kch"),
            teacher: str_field(v, "xm"),
            room: str_field(v, "cdmc"),
            weekday: str_field(v, "xqj"),
            weekday_name: str_field(v, "xqjmc"),
            sections: str_field(v, "jc"),
            weeks: str_field(v, "zcd"),
            credit: f64_field(v, "xf"),
        }
    }
}

impl PracticeCourse {
    fn from_value(v: &Value) -> Self {
        Self {
            name: str_field(v, "kcmc"),
            teacher: str_field(v, "jsxm"),
            weeks: str_field(v, "qsjsz"),
            detail: str_field(v, "qtkcgs"),
        }
    }
}

impl Schedule {
    fn from_response(response: Value) -> Self {
        let xsxx = response.get("xsxx").unwrap_or(&Value::Null);
        Self {
            student: StudentInfo::from_value(xsxx),
            courses: map_array("kbList", &response, Course::from_value),
            practices: map_array("sjkList", &response, PracticeCourse::from_value),
            raw: response,
        }
    }

    pub fn as_json(&self) -> &Value {
        &self.raw
    }

    pub fn into_json(self) -> Value {
        self.raw
    }
}

impl Jwxt {
    pub async fn student_schedule(
        &self,
        year: u32,
        term: Term,
        mode: FetchMode,
    ) -> Result<Cached<Schedule>> {
        let cached = self.student_schedule_json(year, term, mode).await?;
        Ok(cached.map(Schedule::from_response))
    }

    pub async fn student_schedule_json(
        &self,
        year: u32,
        term: Term,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.cached_json(CacheKey::schedule(year, term), mode, async {
            self.ensure_session().await?;
            let year_s = year.to_string();
            let referer = format!(
                "kbcx/xskbcx_cxXskbcxIndex.html?gnmkdm={SCHEDULE_GNMKDM}&layout=default"
            );
            let path = format!("kbcx/xskbcx_cxXsKb.html?gnmkdm={SCHEDULE_GNMKDM}");
            let raw = self
                .post_form(
                    &path,
                    &referer,
                    &[("xnm", year_s.as_str()), ("xqm", term.xqm())],
                )
                .await?;
            parse_json_value(&raw, "schedule")
        })
        .await
    }
}
