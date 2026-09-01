pub mod cache;
pub mod course_select;
pub mod exams;
pub mod grade_details;
pub mod grades;
pub mod makeup;
pub mod profile;
pub mod raw;
pub mod schedule;
pub mod selected;

pub use cache::{
    CacheKey, CacheKind, Cached, FetchMode, JwxtCache, DEFAULT_CACHE_TTL, SCHEDULE_CACHE_TTL,
};
pub use course_select::{
    ClassDetailQuery, SelectableSearch, SelectionContext, SelectionTab, COURSE_SELECT_GNMKDM,
};
pub use raw::{
    QueryResult, f64_field, map_array, map_items, opt_f64_field, parse_json_value,
    parse_profile_fields, parse_query_result, str_field,
};

use reqwest::Client;
use serde_json::Value;
use url::Url;

use cache::SharedCache;
use crate::login::sso::SsoClient;
use crate::utils::{Error, Result};

pub const JWGLXT_BASE: &str = "http://jwglxt.njupt.edu.cn";
pub const JWGLXT_DDLOGIN_SERVICE: &str = "http://jwglxt.njupt.edu.cn/sso/ddlogin";

pub const SCHEDULE_GNMKDM: &str = "N2151";
pub const GRADES_GNMKDM: &str = "N305005";
pub const EXAMS_GNMKDM: &str = "N358105";
pub const MAKEUP_EXAMS_GNMKDM: &str = "N358110";
pub const DEFERRED_EXAMS_GNMKDM: &str = "N358115";
pub const SELECTED_GNMKDM: &str = "N255010";
pub const PROFILE_GNMKDM: &str = "N100801";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Term {
    /// 第一学期（正方 `xqm=3`）
    First,
    /// 第二学期（正方 `xqm=12`）
    Second,
}

impl Term {
    pub fn xqm(self) -> &'static str {
        match self {
            Self::First => "3",
            Self::Second => "12",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Jwxt {
    http: Client,
    cache: SharedCache,
}

impl Jwxt {
    pub fn with_http(http: Client) -> Self {
        Self {
            http,
            cache: SharedCache::default(),
        }
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    /// CacheFirst 时返回已缓存的 raw；NetworkOnly 时始终为 `None`。
    pub(crate) fn cache_lookup(&self, key: &CacheKey, mode: FetchMode) -> Option<Value> {
        match mode {
            FetchMode::NetworkOnly => None,
            FetchMode::CacheFirst => self.cache.get(key),
        }
    }

    pub(crate) fn cache_store(&self, key: CacheKey, value: Value) {
        self.cache.insert(key, value);
    }

    /// lookup → fetch → store；`from_cache` 表示是否命中内存缓存。
    pub(crate) async fn cached_json(
        &self,
        key: CacheKey,
        mode: FetchMode,
        fetch: impl std::future::Future<Output = Result<Value>>,
    ) -> Result<Cached<Value>> {
        if let Some(cached) = self.cache_lookup(&key, mode) {
            return Ok(Cached {
                data: cached,
                from_cache: true,
            });
        }
        let value = fetch.await?;
        self.cache_store(key, value.clone());
        Ok(Cached {
            data: value,
            from_cache: false,
        })
    }

    pub fn has_cache(&self, key: &CacheKey) -> bool {
        self.cache.contains(key)
    }

    pub fn invalidate_cache(&self, key: &CacheKey) -> bool {
        self.cache.invalidate(key)
    }

    pub fn invalidate_cache_kind(&self, kind: CacheKind) {
        self.cache.invalidate_kind(kind);
    }

    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    fn target(&self, path: &str) -> Result<Url> {
        let path = path.trim_start_matches('/');
        Ok(Url::parse(&format!("{JWGLXT_BASE}/{path}"))?)
    }

    async fn get_text(&self, path: &str) -> Result<(Url, String)> {
        let url = self.target(path)?;
        let response = self.http.get(url).send().await?.error_for_status()?;
        let final_url = response.url().clone();
        Ok((final_url, response.text().await?))
    }

    async fn post_form(
        &self,
        path: &str,
        referer_path: &str,
        form: &[(&str, &str)],
    ) -> Result<String> {
        let url = self.target(path)?;
        let referer = self.target(referer_path)?;
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(form.iter().copied())
            .finish();
        Ok(self
            .http
            .post(url)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .header("Referer", referer.as_str())
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    async fn query_items(
        &self,
        path: &str,
        referer_path: &str,
        year: Option<u32>,
        term: Option<Term>,
        extra: &[(&str, &str)],
    ) -> Result<String> {
        let _ = self.get_text(referer_path).await?;
        let year_s = year.map(|y| y.to_string()).unwrap_or_default();
        let xqm = term.map(Term::xqm).unwrap_or("");
        let mut form = vec![
            ("xnm", year_s.as_str()),
            ("xqm", xqm),
            ("_search", "false"),
            ("nd", "1"),
            ("queryModel.showCount", "5000"),
            ("queryModel.currentPage", "1"),
            ("queryModel.sortName", ""),
            ("queryModel.sortOrder", "asc"),
            ("time", "1"),
        ];
        form.extend_from_slice(extra);
        self.post_form(path, referer_path, &form).await
    }

    /// 复用身份中台 TGC，为教务 `ddlogin` 落 CAS 票（service 必须是教务真实 URL）。
    pub async fn ensure_session(&self) -> Result<()> {
        let sso = SsoClient::new(self.http.clone())?;
        let response = sso.goto_service(JWGLXT_DDLOGIN_SERVICE).await?;
        let landed = response.url().as_str();
        if landed.contains("user-login") || landed.contains("login_slogin") {
            return Err(Error::Login(format!(
                "jwxt CAS ticket not accepted, landed on {landed}"
            )));
        }
        if !landed.contains("jwglxt.njupt.edu.cn") {
            return Err(Error::Login(format!(
                "jwxt CAS redirect unexpected: {landed}"
            )));
        }
        Ok(())
    }
}

pub(super) fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
