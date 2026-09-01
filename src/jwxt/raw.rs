use std::collections::BTreeMap;

use serde_json::Value;

use crate::utils::{Error, Result};

use super::truncate;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryResult {
    pub items: Vec<Value>,
    pub response: Value,
}

pub fn parse_query_result(raw: &str, label: &str) -> Result<QueryResult> {
    let response: Value = serde_json::from_str(raw).map_err(|e| {
        Error::Unexpected(format!("{label} json: {e}; body={}", truncate(raw, 200)))
    })?;
    if !response.is_object() {
        return Err(Error::Unexpected(format!(
            "{label}: expected json object; body={}",
            truncate(raw, 200)
        )));
    }
    let items = response
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(QueryResult { items, response })
}

pub fn f64_field(v: &Value, key: &str) -> f64 {
    v.get(key)
        .and_then(|x| match x {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        })
        .unwrap_or(0.0)
}

pub fn opt_f64_field(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(|x| match x {
        Value::Null => None,
        Value::Number(n) => n.as_f64(),
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

pub fn str_field(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| match x {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

pub fn map_items<T, F>(response: &Value, f: F) -> Vec<T>
where
    F: Fn(&Value) -> T,
{
    response
        .get("items")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(f).collect())
        .unwrap_or_default()
}

pub fn map_array<T, F>(key: &str, response: &Value, f: F) -> Vec<T>
where
    F: Fn(&Value) -> T,
{
    response
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(f).collect())
        .unwrap_or_default()
}

pub fn parse_json_value(raw: &str, label: &str) -> Result<Value> {
    serde_json::from_str(raw).map_err(|e| {
        Error::Unexpected(format!("{label} json: {e}; body={}", truncate(raw, 200)))
    })
}

pub fn parse_profile_fields(html: &str) -> BTreeMap<String, String> {
    html.split("form-group")
        .filter_map(|chunk| {
            if !chunk.contains("control-label") {
                return None;
            }
            let after_label = chunk.find("control-label")?;
            let gt = chunk[after_label..].find('>')? + after_label + 1;
            let rest = &chunk[gt..];
            let end = rest.find('<')?;
            let label = rest[..end]
                .trim()
                .trim_end_matches('：')
                .trim_end_matches(':')
                .to_string();
            if label.is_empty() || label.contains('{') || label.len() > 40 {
                return None;
            }
            let static_pos = chunk.find("form-control-static")?;
            let s = &chunk[static_pos..];
            let gt = s.find('>')? + 1;
            let rest = &s[gt..];
            let end = rest.find('<')?;
            let value = rest[..end].trim().to_string();
            Some((label, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_items_array() {
        let raw = r#"{"items":[{"kcmc":"数学"}],"totalResult":1}"#;
        let r = parse_query_result(raw, "test").unwrap();
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.response["totalResult"], 1);
    }

    #[test]
    fn parses_f64_fields() {
        let v = serde_json::json!({"xf": "3.5", "jd": "4.00", "cj": "优"});
        assert!((f64_field(&v, "xf") - 3.5).abs() < f64::EPSILON);
        assert_eq!(opt_f64_field(&v, "jd"), Some(4.0));
        assert_eq!(opt_f64_field(&v, "cj"), None);
    }
}
