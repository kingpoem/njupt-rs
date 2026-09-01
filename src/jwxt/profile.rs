use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{CacheKey, Cached, FetchMode, Jwxt, PROFILE_GNMKDM, parse_profile_fields};
use crate::utils::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentProfile {
    #[serde(skip)]
    fields: BTreeMap<String, String>,
    pub student_id: String,
    pub name: String,
    pub name_pinyin: String,
    pub gender: String,
    pub grade_year: String,
    pub college: String,
    pub major: String,
    pub class_name: String,
    pub study_years: String,
    pub status: String,
    pub in_school: String,
    pub enrollment_date: String,
    pub education_level: String,
    pub email: String,
    pub phone: String,
}

impl StudentProfile {
    fn from_fields(fields: BTreeMap<String, String>) -> Result<Self> {
        let pick = |key: &str| fields.get(key).cloned().unwrap_or_default();
        let student_id = pick("学号");
        if student_id.is_empty() {
            return Err(Error::Unexpected(
                "profile page missing student id".into(),
            ));
        }
        Ok(Self {
            student_id: student_id.clone(),
            name: pick("姓名"),
            name_pinyin: pick("姓名拼音"),
            gender: pick("性别"),
            grade_year: pick("年级"),
            college: pick("学院名称"),
            major: pick("专业名称"),
            class_name: pick("班级名称"),
            study_years: pick("学制"),
            status: pick("学籍状态"),
            in_school: pick("是否在校"),
            enrollment_date: pick("入学日期"),
            education_level: pick("培养层次"),
            email: pick("电子邮箱"),
            phone: pick("固定电话"),
            fields,
        })
    }

    fn fields_from_json(value: Value) -> Result<BTreeMap<String, String>> {
        let obj = match value {
            Value::Object(map) => map,
            _ => {
                return Err(Error::Unexpected(
                    "profile cache: expected json object".into(),
                ));
            }
        };
        Ok(obj
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, s)
            })
            .collect())
    }

    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn as_json(&self) -> Value {
        json!(self.fields)
    }
}

impl Jwxt {
    pub async fn student_profile(&self, mode: FetchMode) -> Result<Cached<StudentProfile>> {
        let cached = self
            .cached_json(CacheKey::profile(), mode, async {
                self.ensure_session().await?;
                let path = format!(
                    "xsxxxggl/xsgrxxwh_cxXsgrxx.html?gnmkdm={PROFILE_GNMKDM}&layout=default"
                );
                let (_, html) = self.get_text(&path).await?;
                let profile = StudentProfile::from_fields(parse_profile_fields(&html))?;
                Ok(profile.as_json())
            })
            .await?;
        let profile = StudentProfile::from_fields(StudentProfile::fields_from_json(cached.data)?)?;
        Ok(Cached {
            data: profile,
            from_cache: cached.from_cache,
        })
    }
}
