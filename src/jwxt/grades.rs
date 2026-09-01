use serde::Serialize;
use serde_json::Value;

use super::{
    CacheKey, Cached, FetchMode, GRADES_GNMKDM, Jwxt, Term, f64_field, map_items, opt_f64_field,
    parse_query_result, str_field,
};
use crate::utils::Result;

#[derive(Debug, Clone, Serialize)]
pub struct Grade {
    pub year: String,
    pub academic_year: String,
    pub term_code: String,
    pub term_name: String,
    pub code: String,
    pub name: String,
    pub name_en: String,
    pub course_nature: String,
    pub course_category: String,
    pub exam_nature: String,
    pub credit: f64,
    pub grade_point: Option<f64>,
    pub credit_grade_point: Option<f64>,
    pub score: String,
    pub score_percent: String,
    pub college: String,
    pub teacher: String,
    pub class_name: String,
    pub assessment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GradeList {
    raw: Value,
    pub items: Vec<Grade>,
}

impl Grade {
    fn from_value(v: &Value) -> Self {
        Self {
            year: str_field(v, "xnm"),
            academic_year: str_field(v, "xnmmc"),
            term_code: str_field(v, "xqm"),
            term_name: str_field(v, "xqmmc"),
            code: str_field(v, "kch"),
            name: str_field(v, "kcmc"),
            name_en: str_field(v, "kcywmc"),
            course_nature: str_field(v, "kcxzmc"),
            course_category: str_field(v, "kclbmc"),
            exam_nature: str_field(v, "ksxz"),
            credit: f64_field(v, "xf"),
            grade_point: opt_f64_field(v, "jd"),
            credit_grade_point: opt_f64_field(v, "xfjd"),
            score: str_field(v, "cj"),
            score_percent: str_field(v, "bfzcj"),
            college: str_field(v, "kkbmmc"),
            teacher: str_field(v, "jsxm"),
            class_name: str_field(v, "jxbmc"),
            assessment: str_field(v, "khfsmc"),
        }
    }
}

impl GradeList {
    fn from_response(response: Value) -> Self {
        let items = map_items(&response, Grade::from_value);
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
    /// `year` / `term` 传 `None` 表示不限学年学期。
    pub async fn student_grades(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<GradeList>> {
        let cached = self.student_grades_json(year, term, mode).await?;
        Ok(cached.map(GradeList::from_response))
    }

    /// 正方原始 JSON 响应（含 items 及分页字段）。
    pub async fn student_grades_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.cached_json(CacheKey::grades(year, term), mode, async {
            self.ensure_session().await?;
            let referer =
                format!("cjcx/cjcx_cxDgXscj.html?gnmkdm={GRADES_GNMKDM}&layout=default");
            let path = format!("cjcx/cjcx_cxDgXscj.html?doType=query&gnmkdm={GRADES_GNMKDM}");
            let raw = self.query_items(&path, &referer, year, term, &[]).await?;
            Ok(parse_query_result(&raw, "grades")?.response)
        })
        .await
    }
}
