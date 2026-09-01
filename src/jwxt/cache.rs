use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use serde_json::Value;

use super::Term;

/// 查询时如何使用内存缓存。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum FetchMode {
    /// 有缓存则直接返回，否则请求正方并写入（默认）。
    #[default]
    CacheFirst,
    /// 忽略缓存，强制请求正方并覆盖写入（下拉刷新）。
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
    Schedule,
    Exams,
    Selected,
    Profile,
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

    pub fn schedule(year: u32, term: Term) -> Self {
        Self::new(CacheKind::Schedule, Some(year), Some(term))
    }

    pub fn exams(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::Exams, year, term)
    }

    pub fn selected(year: Option<u32>, term: Option<Term>) -> Self {
        Self::new(CacheKind::Selected, year, term)
    }

    pub fn profile() -> Self {
        Self::new(CacheKind::Profile, None, None)
    }
}

#[derive(Debug, Default)]
pub struct JwxtCache {
    entries: HashMap<CacheKey, Value>,
}

impl JwxtCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &CacheKey) -> Option<Value> {
        self.entries.get(key).cloned()
    }

    pub fn insert(&mut self, key: CacheKey, value: Value) {
        self.entries.insert(key, value);
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

    pub fn contains(&self, key: &CacheKey) -> bool {
        self.entries.contains_key(key)
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
            .map(|c| c.contains(key))
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
}
