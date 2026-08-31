use serde::Deserialize;
use url::Url;

use crate::login::sso::SsoClient;
use crate::login::Transport;
use crate::utils::{Error, Result};

pub const JWGLXT_BASE: &str = "http://jwglxt.njupt.edu.cn";
pub const SCHEDULE_GNMKDM: &str = "N2151";
pub const JWGLXT_SSO_PATH: &str = "sso/ddlogin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct Jwxt<T> {
    transport: T,
}

impl<T: Transport> Jwxt<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    fn target(&self, path: &str) -> Result<Url> {
        let path = path.trim_start_matches('/');
        Ok(Url::parse(&format!("{JWGLXT_BASE}/{path}"))?)
    }

    async fn get_text(&self, path: &str) -> Result<(Url, String)> {
        let url = self.transport.map_url(&self.target(path)?)?;
        let response = self
            .transport
            .http()
            .get(url)
            .send()
            .await?
            .error_for_status()?;
        let final_url = response.url().clone();
        Ok((final_url, response.text().await?))
    }

    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Result<String> {
        let url = self.transport.map_url(&self.target(path)?)?;
        let referer = self.transport.map_url(&self.target(&format!(
            "kbcx/xskbcx_cxXskbcxIndex.html?gnmkdm={SCHEDULE_GNMKDM}&layout=default"
        ))?)?;
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(form.iter().copied())
            .finish();
        Ok(self
            .transport
            .http()
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

    /// 复用身份中台 TGC，为教务 `ddlogin`（经 Transport 包装）落 CAS 票。
    pub async fn ensure_session(&self) -> Result<()> {
        let service = self.transport.map_url(&self.target(JWGLXT_SSO_PATH)?)?;
        let sso = SsoClient::new(self.transport.http().clone())?;
        let response = sso.goto_service(service.as_str()).await?;
        let landed = response.url().as_str().to_string();
        if landed.contains("user-login") || landed.contains("login_slogin") {
            return Err(Error::Login(format!(
                "jwxt CAS ticket not accepted, landed on {landed}"
            )));
        }

        let (final_url, body) = self
            .get_text(&format!(
                "kbcx/xskbcx_cxXskbcxIndex.html?gnmkdm={SCHEDULE_GNMKDM}&layout=default"
            ))
            .await?;
        if final_url.as_str().contains("login_slogin")
            || body.contains("南邮统一身份认证")
        {
            return Err(Error::Login(
                "jwxt session missing after CAS goto_service".into(),
            ));
        }
        Ok(())
    }

    pub async fn student_schedule(&self, year: u32, term: Term) -> Result<Schedule> {
        self.ensure_session().await?;
        let year_s = year.to_string();
        let path = format!("kbcx/xskbcx_cxXsKb.html?gnmkdm={SCHEDULE_GNMKDM}");
        let raw = self
            .post_form(&path, &[("xnm", year_s.as_str()), ("xqm", term.xqm())])
            .await?;
        let payload: ScheduleApiResponse = serde_json::from_str(&raw).map_err(|e| {
            Error::Unexpected(format!("schedule json: {e}; body={}", truncate(&raw, 200)))
        })?;
        Ok(Schedule::from_api(payload))
    }
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[derive(Debug, Clone, Deserialize)]
struct ScheduleApiResponse {
    xsxx: Option<StudentInfoRaw>,
    #[serde(rename = "kbList", default)]
    kb_list: Vec<CourseRaw>,
    #[serde(rename = "sjkList", default)]
    sjk_list: Vec<PracticeRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct StudentInfoRaw {
    #[serde(rename = "XH")]
    student_id: Option<String>,
    #[serde(rename = "XM")]
    name: Option<String>,
    #[serde(rename = "BJMC")]
    class_name: Option<String>,
    #[serde(rename = "ZYMC")]
    major: Option<String>,
    #[serde(rename = "XNMC")]
    academic_year: Option<String>,
    #[serde(rename = "XNM")]
    year: Option<String>,
    #[serde(rename = "XQM")]
    term_code: Option<String>,
    #[serde(rename = "XQMMC")]
    term_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CourseRaw {
    kcmc: Option<String>,
    kch: Option<String>,
    xm: Option<String>,
    cdmc: Option<String>,
    xqj: Option<String>,
    xqjmc: Option<String>,
    jc: Option<String>,
    zcd: Option<String>,
    xf: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PracticeRaw {
    kcmc: Option<String>,
    jsxm: Option<String>,
    qsjsz: Option<String>,
    qtkcgs: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Schedule {
    pub student: StudentInfo,
    pub courses: Vec<Course>,
    pub practices: Vec<PracticeCourse>,
}

#[derive(Debug, Clone, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct Course {
    pub name: String,
    pub code: String,
    pub teacher: String,
    pub room: String,
    pub weekday: u8,
    pub weekday_name: String,
    pub sections: String,
    pub weeks: String,
    pub credit: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PracticeCourse {
    pub name: String,
    pub teacher: String,
    pub weeks: String,
    pub raw: String,
}

impl Schedule {
    fn from_api(payload: ScheduleApiResponse) -> Self {
        let xs = payload.xsxx.unwrap_or(StudentInfoRaw {
            student_id: None,
            name: None,
            class_name: None,
            major: None,
            academic_year: None,
            year: None,
            term_code: None,
            term_name: None,
        });
        Self {
            student: StudentInfo {
                student_id: xs.student_id.unwrap_or_default(),
                name: xs.name.unwrap_or_default(),
                class_name: xs.class_name.unwrap_or_default(),
                major: xs.major.unwrap_or_default(),
                academic_year: xs.academic_year.unwrap_or_default(),
                year: xs.year.unwrap_or_default(),
                term_code: xs.term_code.unwrap_or_default(),
                term_name: xs.term_name.unwrap_or_default(),
            },
            courses: payload
                .kb_list
                .into_iter()
                .map(|c| Course {
                    name: c.kcmc.unwrap_or_default(),
                    code: c.kch.unwrap_or_default(),
                    teacher: c.xm.unwrap_or_default(),
                    room: c.cdmc.unwrap_or_default(),
                    weekday: c.xqj.and_then(|s| s.parse().ok()).unwrap_or(0),
                    weekday_name: c.xqjmc.unwrap_or_default(),
                    sections: c.jc.unwrap_or_default(),
                    weeks: c.zcd.unwrap_or_default(),
                    credit: c.xf.unwrap_or_default(),
                })
                .collect(),
            practices: payload
                .sjk_list
                .into_iter()
                .map(|c| PracticeCourse {
                    name: c.kcmc.unwrap_or_default(),
                    teacher: c.jsxm.unwrap_or_default(),
                    weeks: c.qsjsz.unwrap_or_default(),
                    raw: c.qtkcgs.unwrap_or_default(),
                })
                .collect(),
        }
    }
}
