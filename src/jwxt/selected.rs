use serde::Serialize;
use serde_json::Value;

use super::{
    CacheKey, Cached, FetchMode, Jwxt, SELECTED_GNMKDM, Term, f64_field, map_items,
    parse_query_result, str_field,
};
use crate::utils::Result;

#[derive(Debug, Clone, Serialize)]
pub struct SelectedCourse {
    pub academic_year: String,
    pub term_name: String,
    pub code: String,
    pub name: String,
    pub college: String,
    pub class_name: String,
    pub teacher: String,
    pub time: String,
    pub place: String,
    pub credit: f64,
    pub weeks: String,
    pub select_type: String,
    pub course_attribute: String,
    pub retake: String,
    pub hours: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectedCourseList {
    raw: Value,
    pub items: Vec<SelectedCourse>,
}

impl SelectedCourse {
    fn from_value(v: &Value) -> Self {
        Self {
            academic_year: str_field(v, "xnmc"),
            term_name: str_field(v, "xqmc"),
            code: str_field(v, "kch"),
            name: str_field(v, "kcmc"),
            college: str_field(v, "kkxymc"),
            class_name: str_field(v, "jxbmc"),
            teacher: str_field(v, "jsxx"),
            time: str_field(v, "sksj"),
            place: str_field(v, "jxdd"),
            credit: f64_field(v, "xf"),
            weeks: str_field(v, "qsjsz"),
            select_type: str_field(v, "xkbjmc"),
            course_attribute: str_field(v, "kcgsmc"),
            retake: str_field(v, "cxbj"),
            hours: str_field(v, "rwzxs"),
        }
    }
}

impl SelectedCourseList {
    fn from_response(response: Value) -> Self {
        let items = map_items(&response, SelectedCourse::from_value);
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
    pub async fn selected_courses(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<SelectedCourseList>> {
        let cached = self.selected_courses_json(year, term, mode).await?;
        Ok(cached.map(SelectedCourseList::from_response))
    }

    pub async fn selected_courses_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.cached_json(CacheKey::selected(year, term), mode, async {
            self.ensure_session().await?;
            let referer =
                format!("xkcx/xkmdcx_cxXkmdcxIndex.html?gnmkdm={SELECTED_GNMKDM}&layout=default");
            let path =
                format!("xkcx/xkmdcx_cxXkmdcxIndex.html?doType=query&gnmkdm={SELECTED_GNMKDM}");
            let raw = self.query_items(&path, &referer, year, term, &[]).await?;
            Ok(parse_query_result(&raw, "selected")?.response)
        })
        .await
    }
}
