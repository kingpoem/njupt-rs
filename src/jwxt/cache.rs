use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use super::Term;

/// 课表默认缓存时长。
pub const SCHEDULE_CACHE_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
/// 其余教务/校园卡数据默认缓存时长。
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(3 * 60 * 60);

/// 查询时如何使用内存缓存。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum FetchMode {
    /// 有未过期缓存则直接返回，否则请求并写入（默认）。
    #[default]
    CacheFirst,
    /// 忽略缓存，强制请求并覆盖写入（下拉刷新）。
    NetworkOnly,
}

/// 带缓存命中信息的查询结果。
#[derive(Debug, Clone, Serialize)]
pub struct Cached<T> {
    pub data: T,
    pub from_cache: bool,
}

impl<T> Cached<T> {
    pub fn into_data(self) -> T {
        self.data
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Cached<U> {
        Cached {
            data: f(self.data),
            from_cache: self.from_cache,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheKind {
    Grades,
    GradeDetails,
    Schedule,
    Exams,
    MakeupExams,
    DeferredExams,
    Selected,
    Profile,
}

impl CacheKind {
    pub fn ttl(self) -> Duration {
        match self {
            Self::Schedule => SCHEDULE_CACHE_TTL,
            _ => DEFAULT_CACHE_TTL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub kind: CacheKind,
    pub year: Option<u32>,
    pub term: Option<Term>,
}

impl CacheKey {
    pub fn new(kind: CacheKind, year: Option<u32>, term: Option<Term>) -> Self {
        Self { kind, year, term }
    }

    pub fn grades(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::Grades, year, term)
    }

    pub fn grade_details(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::GradeDetails, year, term)
    }

    pub fn schedule(year: u32, term: Term) -> Self {
        Self::new(CacheKind::Schedule, Some(year), Some(term))
    }

    pub fn exams(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::Exams, year, term)
    }

    pub fn makeup_exams(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::MakeupExams, year, term)
    }

    pub fn deferred_exams(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::DeferredExams, year, term)
    }

    pub fn selected(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::Selected, year, term)
    }

    pub fn profile() -> Self {
        Self::new(CacheKind::Profile, None, None)
    }

    pub fn ttl(self) -> Duration {
        self.kind.ttl()
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    stored_at: Instant,
}

impl CacheEntry {
    fn fresh(&self, ttl: Duration) -> bool {
        self.stored_at.elapsed() < ttl
    }
}

#[derive(Debug, Default)]
pub struct JwxtCache {
    entries: HashMap<CacheKey, CacheEntry>,
}

impl JwxtCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<Value> {
        let expired = match self.entries.get(key) {
            Some(entry) if entry.fresh(key.ttl()) => {
                return Some(entry.value.clone());
            }
            Some(_) => true,
            None => false,
        };
        if expired {
            self.entries.remove(key);
        }
        None
    }

    pub fn insert(&mut self, key: CacheKey, value: Value) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                stored_at: Instant::now(),
            },
        );
    }

    pub fn invalidate(&mut self, key: &CacheKey) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn invalidate_kind(&mut self, kind: CacheKind) {
        self.entries.retain(|k, _| k.kind != kind);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&mut self, key: &CacheKey) -> bool {
        self.get(key).is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct SharedCache {
    inner: Arc<Mutex<JwxtCache>>,
}

impl SharedCache {
    pub fn get(&self, key: &CacheKey) -> Option<Value> {
        self.inner.lock().ok()?.get(key)
    }

    pub fn insert(&self, key: CacheKey, value: Value) {
        if let Ok(mut cache) = self.inner.lock() {
            cache.insert(key, value);
        }
    }

    pub fn invalidate(&self, key: &CacheKey) -> bool {
        self.inner
            .lock()
            .map(|mut c| c.invalidate(key))
            .unwrap_or(false)
    }

    pub fn invalidate_kind(&self, kind: CacheKind) {
        if let Ok(mut cache) = self.inner.lock() {
            cache.invalidate_kind(kind);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.inner.lock() {
            cache.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().map(|c| c.len()).unwrap_or(0)
    }

    pub fn contains(&self, key: &CacheKey) -> bool {
        self.inner
            .lock()
            .map(|mut c| c.contains(key))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stores_multiple_terms() {
        let mut cache = JwxtCache::new();
        let k1 = CacheKey::schedule(2024, Term::First);
        let k2 = CacheKey::schedule(2025, Term::Second);
        cache.insert(k1, json!({"a": 1}));
        cache.insert(k2, json!({"b": 2}));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&k1).unwrap()["a"], 1);
        assert_eq!(cache.get(&k2).unwrap()["b"], 2);
    }

    #[test]
    fn invalidate_kind_keeps_others() {
        let mut cache = JwxtCache::new();
        cache.insert(CacheKey::grades(Some(2025), Some(Term::Second)), json!(1));
        cache.insert(CacheKey::exams(Some(2025), Some(Term::Second)), json!(2));
        cache.invalidate_kind(CacheKind::Grades);
        assert!(!cache.contains(&CacheKey::grades(Some(2025), Some(Term::Second))));
        assert!(cache.contains(&CacheKey::exams(Some(2025), Some(Term::Second))));
    }

    #[test]
    fn fetch_mode_default_is_cache_first() {
        assert_eq!(FetchMode::default(), FetchMode::CacheFirst);
    }

    #[test]
    fn schedule_ttl_is_month_others_three_hours() {
        assert_eq!(CacheKind::Schedule.ttl(), SCHEDULE_CACHE_TTL);
        assert_eq!(CacheKind::Grades.ttl(), DEFAULT_CACHE_TTL);
        assert_eq!(CacheKind::Profile.ttl(), DEFAULT_CACHE_TTL);
    }

    #[test]
    fn expired_entry_is_ignored() {
        let mut cache = JwxtCache::new();
        let key = CacheKey::grades(Some(2025), Some(Term::First));
        cache.entries.insert(
            key,
            CacheEntry {
                value: json!(1),
                stored_at: Instant::now() - DEFAULT_CACHE_TTL - Duration::from_secs(1),
            },
        );
        assert!(cache.get(&key).is_none());
        assert!(!cache.entries.contains_key(&key));
    }
}
