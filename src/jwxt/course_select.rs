use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::{Jwxt, Term, truncate};
use crate::utils::{Error, Result};

pub const COURSE_SELECT_GNMKDM: &str = "N253512";

const INDEX_PATH: &str = "xsxk/zzxkyzb_cxZzxkYzbIndex.html";
const DISPLAY_PATH: &str = "xsxk/zzxkyzb_cxZzxkYzbDisplay.html";
const PART_DISPLAY_PATH: &str = "xsxk/zzxkyzb_cxZzxkYzbPartDisplay.html";
const CLASS_DETAIL_PATH: &str = "xsxk/zzxkyzbjk_cxJxbWithKchZzxkYzb.html";
const CHOSEN_PATH: &str = "xsxk/zzxkyzb_cxZzxkYzbChoosedDisplay.html";

#[derive(Debug, Clone, Serialize)]
pub struct SelectionTab {
    pub name: String,
    pub kklxdm: String,
    pub xkkz_id: String,
}

/// 网上选课页上下文：隐藏域 + 选课板块 tab。
#[derive(Debug, Clone, Serialize)]
pub struct SelectionContext {
    pub fields: BTreeMap<String, String>,
    pub tabs: Vec<SelectionTab>,
}

impl SelectionContext {
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn tab_by_kklxdm(&self, kklxdm: &str) -> Option<&SelectionTab> {
        self.tabs.iter().find(|t| t.kklxdm == kklxdm)
    }
}

#[derive(Debug, Clone)]
pub struct SelectableSearch {
    pub year: u32,
    pub term: Term,
    pub kklxdm: String,
    pub filter: Option<String>,
    pub page_start: u32,
    pub page_end: u32,
    /// `yl_list[0]=1`：仅看有余量
    pub only_available: bool,
}

impl Default for SelectableSearch {
    fn default() -> Self {
        Self {
            year: 0,
            term: Term::First,
            kklxdm: "01".into(),
            filter: None,
            page_start: 1,
            page_end: 10,
            only_available: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassDetailQuery {
    pub year: u32,
    pub term: Term,
    pub kklxdm: String,
    pub kch_id: String,
    pub xkkz_id: String,
}

impl Jwxt {
    fn select_referer(&self) -> String {
        format!("{INDEX_PATH}?gnmkdm={COURSE_SELECT_GNMKDM}&layout=default")
    }

    fn select_path(base: &str) -> String {
        format!("{base}?gnmkdm={COURSE_SELECT_GNMKDM}")
    }

    /// 拉取网上选课首页上下文（只读）。非选课开放期可能返回无权限/空 tab。
    pub async fn selection_context(&self) -> Result<SelectionContext> {
        self.ensure_session().await?;
        let referer = self.select_referer();
        let (_, html) = self.get_text(&referer).await?;
        reject_permission_page(&html, "selection_context")?;

        let mut fields = parse_hidden_inputs(&html);
        let tabs = parse_selection_tabs(&html);

        if let Some(tab) = tabs.first() {
            let form = [
                ("xkkz_id", tab.xkkz_id.as_str()),
                ("xszxzt", "1"),
                ("kspage", "0"),
            ];
            let display = self
                .post_form(&Self::select_path(DISPLAY_PATH), &referer, &form)
                .await?;
            if !looks_like_error_page(&display) {
                for (k, v) in parse_hidden_inputs(&display) {
                    fields.entry(k).or_insert(v);
                }
            }
        }

        Ok(SelectionContext { fields, tabs })
    }

    /// 搜索可选课程列表（`PartDisplay`，只读）。
    pub async fn search_selectable_courses(
        &self,
        ctx: &SelectionContext,
        query: &SelectableSearch,
    ) -> Result<Value> {
        self.ensure_session().await?;
        let referer = self.select_referer();
        let mut form = base_student_form(ctx);
        form.insert("xkxnm".into(), query.year.to_string());
        form.insert("xkxqm".into(), query.term.xqm().into());
        form.insert("kklxdm".into(), query.kklxdm.clone());
        form.insert("kspage".into(), query.page_start.to_string());
        form.insert("jspage".into(), query.page_end.to_string());
        if let Some(filter) = &query.filter {
            if !filter.is_empty() {
                form.insert("filter_list[0]".into(), filter.clone());
            }
        }
        if query.only_available {
            form.insert("yl_list[0]".into(), "1".into());
        }

        let raw = self
            .post_form(
                &Self::select_path(PART_DISPLAY_PATH),
                &referer,
                &form_pairs(&form),
            )
            .await?;
        parse_json_or_err(&raw, "search_selectable_courses")
    }

    /// 按课程号展开教学班详情 / 余量（只读）。
    pub async fn selectable_class_details(
        &self,
        ctx: &SelectionContext,
        query: &ClassDetailQuery,
    ) -> Result<Value> {
        self.ensure_session().await?;
        let referer = self.select_referer();
        let mut form = base_student_form(ctx);
        form.insert("xkxnm".into(), query.year.to_string());
        form.insert("xkxqm".into(), query.term.xqm().into());
        form.insert("kklxdm".into(), query.kklxdm.clone());
        form.insert("kch_id".into(), query.kch_id.clone());
        form.insert("xkkz_id".into(), query.xkkz_id.clone());

        let raw = self
            .post_form(
                &Self::select_path(CLASS_DETAIL_PATH),
                &referer,
                &form_pairs(&form),
            )
            .await?;
        parse_json_or_err(&raw, "selectable_class_details")
    }

    /// 选课模块内「已选」展示（与 `selected_courses` 名单查询不同入口，只读）。
    pub async fn selection_chosen(
        &self,
        ctx: &SelectionContext,
        year: u32,
        term: Term,
    ) -> Result<Value> {
        self.ensure_session().await?;
        let referer = self.select_referer();
        let mut form = base_student_form(ctx);
        form.insert("xkxnm".into(), year.to_string());
        form.insert("xkxqm".into(), term.xqm().into());

        let raw = self
            .post_form(
                &Self::select_path(CHOSEN_PATH),
                &referer,
                &form_pairs(&form),
            )
            .await?;
        parse_json_or_err(&raw, "selection_chosen")
    }
}

fn base_student_form(ctx: &SelectionContext) -> BTreeMap<String, String> {
    let keys = [
        "bklx_id",
        "xqh_id",
        "zyfx_id",
        "njdm_id",
        "bh_id",
        "xbm",
        "xslbdm",
        "ccdm",
        "xsbj",
        "zyh_id",
        "jg_id",
        "xkly",
        "rwlx",
        "kkbk",
        "kkbkdj",
        "sfkkjyxdxnxq",
        "sfkknj",
        "sfkkzy",
        "kzybkxy",
        "sfznkx",
        "zdkxms",
        "sfkxq",
        "sfkcfx",
        "sfkgbcx",
        "sfrxtgkcxd",
        "tykczgxdcs",
        "rlkz",
        "xkzgbj",
    ];
    let mut form = BTreeMap::new();
    for key in keys {
        if let Some(v) = ctx.field(key) {
            form.insert(key.into(), v.into());
        }
    }
    form
}

fn form_pairs(form: &BTreeMap<String, String>) -> Vec<(&str, &str)> {
    form.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
}

fn parse_json_or_err(raw: &str, label: &str) -> Result<Value> {
    let trimmed = raw.trim_start();
    if looks_like_error_page(raw) || !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return Err(Error::Unexpected(format!(
            "{label}: expected json; body={}",
            truncate(raw, 200)
        )));
    }
    serde_json::from_str(raw).map_err(|e| {
        Error::Unexpected(format!("{label} json: {e}; body={}", truncate(raw, 200)))
    })
}

fn reject_permission_page(html: &str, label: &str) -> Result<()> {
    if looks_like_error_page(html) {
        return Err(Error::Unexpected(format!(
            "{label}: selection page unavailable; body={}",
            truncate(html, 200)
        )));
    }
    Ok(())
}

fn looks_like_error_page(body: &str) -> bool {
    body.contains("无功能权限")
        || body.contains("用户登录")
        || body.contains("方法未定义")
        || body.contains("不存在的功能")
}

fn parse_hidden_inputs(html: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let lower = html.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find("<input") {
        let start = search_from + rel;
        let from_tag = &html[start..];
        let end = from_tag.find('>').map(|i| i + 1).unwrap_or(from_tag.len());
        let tag = &from_tag[..end];
        search_from = start + end;

        let tag_l = tag.to_ascii_lowercase();
        if !tag_l.contains("hidden") {
            continue;
        }
        let Some(name) = html_attr(tag, "name") else {
            continue;
        };
        let value = html_attr(tag, "value").unwrap_or_default();
        map.insert(name, value);
    }
    map
}

fn parse_selection_tabs(html: &str) -> Vec<SelectionTab> {
    let mut tabs = Vec::new();
    let mut offset = 0;
    while let Some(rel) = html[offset..].find("onclick=") {
        let abs = offset + rel;
        let after = &html[abs + "onclick=".len()..];
        let (quoted, consumed) = match after.chars().next() {
            Some('"') => {
                let body = &after[1..];
                match body.find('"') {
                    Some(e) => (&body[..e], 1 + e + 1),
                    None => break,
                }
            }
            Some('\'') => {
                let body = &after[1..];
                match body.find('\'') {
                    Some(e) => (&body[..e], 1 + e + 1),
                    None => break,
                }
            }
            _ => {
                offset = abs + "onclick=".len();
                continue;
            }
        };
        offset = abs + "onclick=".len() + consumed;

        if !quoted.contains("queryCourse") && !quoted.contains("queryCourseByKklxdm") {
            continue;
        }
        let args = single_quoted_args(quoted);
        if args.len() < 2 {
            continue;
        }
        let kklxdm = args[0].clone();
        let xkkz_id = args[1].clone();
        if kklxdm.is_empty() || xkkz_id.is_empty() {
            continue;
        }
        let name = nearest_tab_label(html, abs).unwrap_or_default();
        if tabs
            .iter()
            .any(|t: &SelectionTab| t.kklxdm == kklxdm && t.xkkz_id == xkkz_id)
        {
            continue;
        }
        tabs.push(SelectionTab {
            name,
            kklxdm,
            xkkz_id,
        });
    }
    tabs
}

fn single_quoted_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find('\'') {
        let body = &rest[start + 1..];
        let Some(end) = body.find('\'') else {
            break;
        };
        out.push(body[..end].to_string());
        rest = &body[end + 1..];
    }
    out
}

fn nearest_tab_label(html: &str, onclick_abs: usize) -> Option<String> {
    let before = &html[..onclick_abs];
    let open = before.rfind('<')?;
    let tag = &html[open..];
    let close = tag.find('>')?;
    let after_open = &tag[close + 1..];
    let end = after_open.find('<')?;
    let text = after_open[..end].trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn html_attr(tag: &str, key: &str) -> Option<String> {
    let patterns = [
        format!("{key}=\""),
        format!("{key}='"),
        format!("{}=\"", key.to_ascii_uppercase()),
        format!("{}='", key.to_ascii_uppercase()),
    ];
    for (i, pat) in patterns.iter().enumerate() {
        if let Some(pos) = tag.find(pat) {
            let rest = &tag[pos + pat.len()..];
            let quote = if i % 2 == 0 { '"' } else { '\'' };
            let end = rest.find(quote).unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hidden_and_tabs() {
        let html = r#"
        <input type="hidden" name="njdm_id" value="2023"/>
        <input type="hidden" name="zyh_id" id="zyh" value="0801"/>
        <a role="tab" onclick="queryCourse(this,'01','AAA111')">主修课程</a>
        <a role="tab" onclick="queryCourse(this,'10','BBB222')">通识选修</a>
        "#;
        let fields = parse_hidden_inputs(html);
        assert_eq!(fields.get("njdm_id").unwrap(), "2023");
        assert_eq!(fields.get("zyh_id").unwrap(), "0801");
        let tabs = parse_selection_tabs(html);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].kklxdm, "01");
        assert_eq!(tabs[0].xkkz_id, "AAA111");
        assert_eq!(tabs[0].name, "主修课程");
        assert_eq!(tabs[1].kklxdm, "10");
    }
}
