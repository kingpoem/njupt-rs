use serde::Serialize;
use serde_json::Value;

use super::{
    CacheKey, Cached, EXAMS_GNMKDM, FetchMode, Jwxt, Term, map_items, parse_query_result, str_field,
};
use crate::utils::Result;

#[derive(Debug, Clone, Serialize)]
pub struct Exam {
    pub year: String,
    pub academic_year: String,
    pub term_code: String,
    pub term_name: String,
    pub code: String,
    pub name: String,
    pub exam_name: String,
    pub date: String,
    pub time: String,
    pub room: String,
    pub seat: String,
    pub campus: String,
    pub class_name: String,
    pub teacher: String,
    pub college: String,
    pub exam_method: String,
    pub assessment: String,
    pub retake: String,
    pub remark: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExamList {
    raw: Value,
    pub items: Vec<Exam>,
}

impl Exam {
    fn from_value(v: &Value) -> Self {
        Self {
            year: str_field(v, "xnm"),
            academic_year: str_field(v, "xnmc"),
            term_code: str_field(v, "xqm"),
            term_name: str_field(v, "xqmmc"),
            code: str_field(v, "kch"),
            name: str_field(v, "kcmc"),
            exam_name: str_field(v, "ksmc"),
            date: str_field(v, "ksrq"),
            time: str_field(v, "kssj"),
            room: str_field(v, "cdmc"),
            seat: str_field(v, "zwh"),
            campus: str_field(v, "cdxqmc"),
            class_name: str_field(v, "jxbmc"),
            teacher: str_field(v, "jsxx"),
            college: str_field(v, "kkxy"),
            exam_method: str_field(v, "ksfs"),
            assessment: str_field(v, "khfs"),
            retake: str_field(v, "cxbj"),
            remark: str_field(v, "bzxx"),
        }
    }
}

impl ExamList {
    fn from_response(response: Value) -> Self {
        let items = map_items(&response, Exam::from_value);
        Self { raw: response, items }
    }

    pub fn as_json(&self) -> &Value {
        &self.raw
    }

    pub fn into_json(self) -> Value {
        self.raw
    }
}

impl Jwxt {
    /// `year` / `term` 传 `None` 表示不限学年学期。无排考时 items 为空。
    pub async fn student_exams(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<ExamList>> {
        let cached = self.student_exams_json(year, term, mode).await?;
        Ok(cached.map(ExamList::from_response))
    }

    pub async fn student_exams_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.cached_json(CacheKey::exams(year, term), mode, async {
            self.ensure_session().await?;
            let referer =
                format!("kwgl/kscx_cxXsksxxIndex.html?gnmkdm={EXAMS_GNMKDM}&layout=default");
            let path =
                format!("kwgl/kscx_cxXsksxxIndex.html?doType=query&gnmkdm={EXAMS_GNMKDM}");
            let raw = self
                .query_items(
                    &path,
                    &referer,
                    year,
                    term,
                    &[
                        ("ksmcdmb_id", ""),
                        ("kch", ""),
                        ("kc", ""),
                        ("ksrq", ""),
                    ],
                )
                .await?;
            Ok(parse_query_result(&raw, "exams")?.response)
        })
        .await
    }
}
