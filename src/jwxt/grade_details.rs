use serde::Serialize;
use serde_json::{json, Value};

use super::{
    CacheKey, Cached, FetchMode, GRADES_GNMKDM, Jwxt, Term, f64_field, map_items, parse_query_result,
    str_field,
};
use crate::utils::{Error, Result};

#[derive(Debug, Clone, Serialize)]
pub struct GradeDetail {
    pub year: String,
    pub academic_year: String,
    pub term_name: String,
    pub code: String,
    pub name: String,
    pub class_name: String,
    pub credit: f64,
    pub component: String,
    pub score: String,
    pub weight: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GradeDetailList {
    raw: Value,
    pub items: Vec<GradeDetail>,
}

impl GradeDetail {
    fn from_value(v: &Value) -> Self {
        Self {
            year: str_field(v, "xnm"),
            academic_year: str_field(v, "xnmmc"),
            term_name: str_field(v, "xqmmc"),
            code: str_field(v, "kch"),
            name: str_field(v, "kcmc"),
            class_name: str_field(v, "jxbmc"),
            credit: f64_field(v, "xf"),
            component: str_field(v, "xmblmc"),
            score: str_field(v, "xmcj"),
            weight: str_field(v, "xmbz"),
        }
    }
}

impl GradeDetailList {
    fn from_response(response: Value) -> Self {
        let items = map_items(&response, GradeDetail::from_value);
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
    /// 成绩分项明细（成绩查询弹窗 `cjcx_cxCjxqGjh`）。
    ///
    /// 南邮当前多数课程只公布「总评」，平时/期末分项需教师录入后才会出现。
    pub async fn grade_details(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<GradeDetailList>> {
        let cached = self.grade_details_json(year, term, mode).await?;
        Ok(cached.map(GradeDetailList::from_response))
    }

    pub async fn grade_details_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.cached_json(CacheKey::grade_details(year, term), mode, async {
            self.ensure_session().await?;
            self.fetch_grade_details_raw(year, term).await
        })
        .await
    }

    async fn fetch_grade_details_raw(
        &self,
        year: Option<u32>,
        term: Option<Term>,
    ) -> Result<Value> {
        let grades_referer =
            format!("cjcx/cjcx_cxDgXscj.html?gnmkdm={GRADES_GNMKDM}&layout=default");
        let grades_path =
            format!("cjcx/cjcx_cxDgXscj.html?doType=query&gnmkdm={GRADES_GNMKDM}");
        let grades_raw = self
            .query_items(&grades_path, &grades_referer, year, term, &[])
            .await?;
        let grades = parse_query_result(&grades_raw, "grades_for_details")?.response;
        let courses = grades
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut all_items = Vec::new();
        for course in &courses {
            let jxb_id = str_field(course, "jxb_id");
            if jxb_id.is_empty() {
                continue;
            }
            let rows = self.fetch_course_cjxq(course).await?;
            for (component, weight, score) in rows {
                let mut item = course.clone();
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("xmblmc".into(), Value::String(component));
                    obj.insert("xmbz".into(), Value::String(weight));
                    obj.insert("xmcj".into(), Value::String(score));
                }
                all_items.push(item);
            }
        }

        Ok(json!({
            "items": all_items,
            "totalResult": all_items.len(),
        }))
    }

    async fn fetch_course_cjxq(&self, course: &Value) -> Result<Vec<(String, String, String)>> {
        let jxb_id = str_field(course, "jxb_id");
        let xnm = str_field(course, "xnm");
        let xqm = str_field(course, "xqm");
        let xh_id = str_field(course, "xh_id");
        let kcmc = str_field(course, "kcmc");
        let cj = str_field(course, "cj");

        let referer = format!("cjcx/cjcx_cxDgXscj.html?gnmkdm={GRADES_GNMKDM}&layout=default");
        let path = format!("cjcx/cjcx_cxCjxqGjh.html?gnmkdm={GRADES_GNMKDM}");
        let html = self
            .post_form(
                &path,
                &referer,
                &[
                    ("jxb_id", &jxb_id),
                    ("xnm", &xnm),
                    ("xqm", &xqm),
                    ("xh_id", &xh_id),
                    ("kcmc", &kcmc),
                    ("cj", &cj),
                ],
            )
            .await?;

        if html.contains("错误") || html.contains("警告") || html.contains("方法未定义") {
            return Err(Error::Unexpected(format!(
                "grade_details: cjxq error for jxb_id={jxb_id}"
            )));
        }

        Ok(parse_cjxq_rows(&html))
    }
}

fn parse_cjxq_rows(html: &str) -> Vec<(String, String, String)> {
    let Some(start) = html.find("<tbody>") else {
        return Vec::new();
    };
    let rest = &html[start..];
    let Some(end) = rest.find("</tbody>") else {
        return Vec::new();
    };
    let tbody = &rest[..end];

    let mut rows = Vec::new();
    let mut search = tbody;
    while let Some(tr_at) = search.find("<tr>") {
        let after_tr = &search[tr_at + 4..];
        let Some(tr_end) = after_tr.find("</tr>") else {
            break;
        };
        let tr = &after_tr[..tr_end];
        let tds = extract_tds(tr);
        if tds.len() >= 3 {
            let name = tds[0].clone();
            if !name.is_empty() && !name.contains("成绩分项") {
                rows.push((name, tds[1].clone(), tds[2].clone()));
            }
        }
        search = &after_tr[tr_end + 5..];
    }
    rows
}

fn extract_tds(tr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut s = tr;
    while let Some(td_at) = s.find("<td") {
        let after = &s[td_at..];
        let Some(gt) = after.find('>') else {
            break;
        };
        let content_start = &after[gt + 1..];
        let Some(close) = content_start.find("</td>") else {
            break;
        };
        out.push(strip_tags(&content_start[..close]));
        s = &content_start[close + 5..];
    }
    out
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace('\u{a0}', " ")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::parse_cjxq_rows;

    #[test]
    fn parses_total_only_table() {
        let html = r#"
        <tbody>
          <tr>
            <td valign="middle">【 总评 】</td>
            <td valign="middle">&nbsp;</td>
            <td valign="middle">90&nbsp;</td>
          </tr>
        </tbody>
        "#;
        let rows = parse_cjxq_rows(html);
        assert_eq!(rows, vec![("【 总评 】".into(), "".into(), "90".into())]);
    }

    #[test]
    fn parses_multi_components() {
        let html = r#"
        <tbody>
          <tr><td>平时</td><td>30%</td><td>85</td></tr>
          <tr><td>期末</td><td>70%</td><td>92</td></tr>
          <tr><td>【 总评 】</td><td></td><td>90</td></tr>
        </tbody>
        "#;
        let rows = parse_cjxq_rows(html);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], ("平时".into(), "30%".into(), "85".into()));
        assert_eq!(rows[1].0, "期末");
    }
}
