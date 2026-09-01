use super::exams::{Exam, ExamList};
use super::{
    CacheKey, Cached, DEFERRED_EXAMS_GNMKDM, FetchMode, Jwxt, MAKEUP_EXAMS_GNMKDM, Term,
    parse_query_result,
};
use crate::utils::Result;
use serde_json::Value;

impl Jwxt {
    /// 补考安排。`year` / `term` 传 `None` 表示不限。无安排时 items 为空。
    pub async fn makeup_exams(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<ExamList>> {
        let cached = self.makeup_exams_json(year, term, mode).await?;
        Ok(cached.map(ExamList::from_response))
    }

    pub async fn makeup_exams_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.query_exam_like(
            CacheKey::makeup_exams(year, term),
            MAKEUP_EXAMS_GNMKDM,
            year,
            term,
            mode,
            "makeup_exams",
        )
        .await
    }

    /// 缓考安排。`year` / `term` 传 `None` 表示不限。无安排时 items 为空。
    pub async fn deferred_exams(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<ExamList>> {
        let cached = self.deferred_exams_json(year, term, mode).await?;
        Ok(cached.map(ExamList::from_response))
    }

    pub async fn deferred_exams_json(
        &self,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
    ) -> Result<Cached<Value>> {
        self.query_exam_like(
            CacheKey::deferred_exams(year, term),
            DEFERRED_EXAMS_GNMKDM,
            year,
            term,
            mode,
            "deferred_exams",
        )
        .await
    }

    async fn query_exam_like(
        &self,
        key: CacheKey,
        gnmkdm: &str,
        year: Option<u32>,
        term: Option<Term>,
        mode: FetchMode,
        label: &str,
    ) -> Result<Cached<Value>> {
        self.cached_json(key, mode, async {
            self.ensure_session().await?;
            let referer =
                format!("kwgl/kscx_cxXsksxxIndex.html?gnmkdm={gnmkdm}&layout=default");
            let path = format!("kwgl/kscx_cxXsksxxIndex.html?doType=query&gnmkdm={gnmkdm}");
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
            Ok(parse_query_result(&raw, label)?.response)
        })
        .await
    }
}

pub type MakeupExam = Exam;
pub type MakeupExamList = ExamList;
