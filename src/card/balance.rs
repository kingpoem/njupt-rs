use serde::Serialize;
use serde_json::Value;

use super::{Card, PORTAL_BASE, YKT_BALANCE_PATH};
use crate::jwxt::{Cached, FetchMode};
use crate::utils::{Error, Result};


#[derive(Debug, Clone, Serialize)]
pub struct CardBalance {
    pub amount: f64,
    pub display: String,
    #[serde(skip)]
    raw: Value,
}

impl CardBalance {
    fn from_response(raw: Value) -> Result<Self> {
        let success = raw
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let code = raw.get("code").and_then(Value::as_i64).unwrap_or(-1);
        if !success || code != 200 {
            return Err(Error::Unexpected(format!(
                "card balance: success={success} code={code} body={raw}"
            )));
        }
        let display = raw
            .get("result")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| raw.get("message").and_then(Value::as_str).map(str::trim))
            .ok_or_else(|| Error::Unexpected("card balance: missing result".into()))?
            .to_string();
        let amount = parse_yuan_amount(&display)?;
        Ok(Self {
            amount,
            display,
            raw,
        })
    }

    pub fn as_json(&self) -> &Value {
        &self.raw
    }

    pub fn into_json(self) -> Value {
        self.raw
    }
}

fn parse_yuan_amount(display: &str) -> Result<f64> {
    let num = display
        .trim()
        .trim_end_matches('元')
        .trim()
        .replace(',', "");
    num.parse::<f64>().map_err(|_| {
        Error::Unexpected(format!("card balance: cannot parse amount from `{display}`"))
    })
}

impl Card {
    pub async fn balance(&self, mode: FetchMode) -> Result<Cached<CardBalance>> {
        let cached = self.balance_json(mode).await?;
        Ok(Cached {
            data: CardBalance::from_response(cached.data)?,
            from_cache: cached.from_cache,
        })
    }

    pub async fn balance_json(&self, mode: FetchMode) -> Result<Cached<Value>> {
        if matches!(mode, FetchMode::CacheFirst) {
            if let Some(cached) = self.balance_cache_get() {
                return Ok(Cached {
                    data: cached,
                    from_cache: true,
                });
            }
        }

        self.ensure_portal().await?;
        let value = self.fetch_balance_raw().await?;
        self.balance_cache_store(value.clone());
        Ok(Cached {
            data: value,
            from_cache: false,
        })
    }

    async fn fetch_balance_raw(&self) -> Result<Value> {
        let url = format!("{PORTAL_BASE}{YKT_BALANCE_PATH}");
        let raw: Value = self
            .http()
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_yuan_amount, CardBalance};
    use serde_json::json;

    #[test]
    fn parses_yuan_display() {
        assert_eq!(parse_yuan_amount("123.75元").unwrap(), 123.75);
        assert_eq!(parse_yuan_amount(" 0元 ").unwrap(), 0.0);
        assert_eq!(parse_yuan_amount("1,234.5元").unwrap(), 1234.5);
    }

    #[test]
    fn rejects_bad_display() {
        assert!(parse_yuan_amount("无").is_err());
    }

    #[test]
    fn from_response_ok() {
        let raw = json!({
            "success": true,
            "code": 200,
            "message": "12.00元",
            "result": "12.00元"
        });
        let b = CardBalance::from_response(raw).unwrap();
        assert_eq!(b.amount, 12.0);
        assert_eq!(b.display, "12.00元");
    }

    #[test]
    fn from_response_rejects_failure() {
        let raw = json!({"success": false, "code": 500, "result": "-"});
        assert!(CardBalance::from_response(raw).is_err());
    }
}
