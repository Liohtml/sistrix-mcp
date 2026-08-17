//! HTTP client for the SISTRIX API.
//!
//! All requests go through `POST https://api.sistrix.com/<method>` with
//! form-encoded parameters so the `api_key` never appears in URLs or server
//! access logs. Responses are requested as JSON (`format=json`). Transient
//! failures (429/5xx, timeouts, connection errors) are retried with
//! exponential backoff — SISTRIX allows 300 requests/minute with at least
//! 300 ms between requests.

use std::sync::Arc;
use std::time::Duration;

use reqwest::header::HeaderValue;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, warn};
use url::Url;

use crate::config::Config;

const MAX_ATTEMPTS: u32 = 3;
const BACKOFF_BASE_MS: u64 = 400;
/// SISTRIX requires at least 300 ms between requests (300 requests/minute).
const MIN_REQUEST_SPACING: Duration = Duration::from_millis(300);
/// Cap error bodies quoted back to the model.
const ERROR_BODY_PREVIEW: usize = 500;

#[derive(Debug, Error)]
pub enum SistrixError {
    #[error("SISTRIX API error{}: {message}{}", code_suffix(.code), hint_suffix(.hint))]
    Api {
        code: Option<i64>,
        message: String,
        hint: Option<&'static str>,
    },

    #[error("HTTP {status} from SISTRIX: {body}{}", hint_suffix(.hint))]
    Http {
        status: StatusCode,
        body: String,
        hint: Option<&'static str>,
    },

    #[error(
        "could not reach the SISTRIX API: {0}. Check your network and that api.sistrix.com is up."
    )]
    Network(String),

    #[error("SISTRIX returned a non-JSON response: {0}")]
    InvalidResponse(String),
}

fn code_suffix(code: &Option<i64>) -> String {
    code.map(|c| format!(" (code {c})")).unwrap_or_default()
}

fn hint_suffix(hint: &Option<&'static str>) -> String {
    hint.map(|h| format!(" Hint: {h}")).unwrap_or_default()
}

/// Hints for the documented SISTRIX error codes the model can act on.
fn hint_for_code(code: i64) -> Option<&'static str> {
    match code {
        100 => Some("check that SISTRIX_API_KEY is valid (app.sistrix.com/account/api)"),
        200 | 3501 => Some(
            "the account is out of API credits — check the balance with sistrix_credits; \
             credits refill weekly",
        ),
        403 | 5000 => Some(
            "the booked SISTRIX package has no access to this function — API access starts \
             with the Plus package",
        ),
        404 => Some("the method name is unknown — see https://www.sistrix.com/api/"),
        429 => Some("rate limit is 300 requests/minute — slow down and retry"),
        1000 => Some(
            "the query matched no data — this is an empty result, not a failure; \
             try another country index or a broader scope",
        ),
        1001 => Some("dates must be in YYYY-MM-DD format and within the available data range"),
        1002 | 1003 | 4007 => Some("check the parameters against https://www.sistrix.com/api/"),
        1004 => Some("this SISTRIX function was retired — check the docs for a replacement"),
        2000 | 2001 => {
            Some("check the domain spelling; the domain may not be in the SISTRIX database")
        }
        3000 | 3001 => Some("the keyword is not in the SISTRIX database (or has no SERP history)"),
        3502 => Some("too many parameters for this query — remove some filters"),
        4000..=4006 => {
            Some("check the project hash — list projects via report=projects/list first")
        }
        5001 => Some("this action requires unlimited API access (SISTRIX Premium)"),
        _ => None,
    }
}

/// Client for the SISTRIX API.
#[derive(Debug, Clone)]
pub struct SistrixClient {
    http: Client,
    base: Url,
    api_key: Option<String>,
    /// Start time of the last request — clones share it, so concurrent tool
    /// calls are spaced out too.
    last_request: Arc<Mutex<Option<Instant>>>,
}

impl SistrixClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let mut base = config.api_url.clone();
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }

        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(
                HeaderValue::try_from(format!("sistrix-mcp/{}", env!("CARGO_PKG_VERSION")))
                    .expect("static user agent is valid"),
            )
            .build()?;

        Ok(Self {
            http,
            base,
            api_key: config.api_key.clone(),
            last_request: Arc::new(Mutex::new(None)),
        })
    }

    /// Enforce the documented minimum spacing between request starts.
    async fn pace(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < MIN_REQUEST_SPACING {
                tokio::time::sleep(MIN_REQUEST_SPACING - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    /// Call a SISTRIX API method (e.g. `domain.overview`) with query parameters.
    pub async fn call(
        &self,
        method: &str,
        params: &[(String, String)],
    ) -> Result<Value, SistrixError> {
        let endpoint = self
            .base
            .join(method)
            .map_err(|e| SistrixError::Network(e.to_string()))?;

        let mut form: Vec<(&str, &str)> = vec![("format", "json")];
        if let Some(key) = &self.api_key {
            form.push(("api_key", key));
        }
        for (k, v) in params {
            form.push((k.as_str(), v.as_str()));
        }

        debug!(method, params = params.len(), "calling SISTRIX API");

        let mut attempt = 0;
        loop {
            attempt += 1;
            self.pace().await;
            match self.send_once(&endpoint, &form).await {
                Err(err) if attempt < MAX_ATTEMPTS && is_retryable(&err) => {
                    let delay = BACKOFF_BASE_MS * 2u64.pow(attempt - 1);
                    warn!(
                        method,
                        attempt,
                        delay_ms = delay,
                        "retrying after transient error: {err}"
                    );
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                result => return result,
            }
        }
    }

    async fn send_once(
        &self,
        endpoint: &Url,
        form: &[(&str, &str)],
    ) -> Result<Value, SistrixError> {
        let response = self
            .http
            .post(endpoint.clone())
            .form(form)
            .send()
            .await
            .map_err(|e| SistrixError::Network(self.scrub(&e.to_string())))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| SistrixError::Network(self.scrub(&e.to_string())))?;

        if !status.is_success() {
            let hint = match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    Some("check that SISTRIX_API_KEY is valid (app.sistrix.com/account/api)")
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    Some("rate limit is 300 requests/minute — slow down and retry")
                }
                _ => None,
            };
            return Err(SistrixError::Http {
                status,
                body: self.scrub(preview(&text)),
                hint,
            });
        }

        let json: Value = serde_json::from_str(&text)
            .map_err(|_| SistrixError::InvalidResponse(self.scrub(preview(&text))))?;

        // SISTRIX signals errors in-band, e.g.
        // {"status":"fail","error":[{"error_code":100,"error_message":"wrong api key"}]}.
        if let Some(err) = extract_api_error(&json) {
            return Err(SistrixError::Api {
                code: err.0,
                message: self.scrub(&err.1),
                hint: err.0.and_then(hint_for_code),
            });
        }

        Ok(json)
    }

    /// Never let the API key leak into error messages shown to the model or logs.
    fn scrub(&self, text: &str) -> String {
        match &self.api_key {
            Some(k) if !k.is_empty() => text.replace(k.as_str(), "[REDACTED]"),
            _ => text.to_string(),
        }
    }
}

/// Detect an in-band SISTRIX error and pull out `(code, message)`.
///
/// The docs don't specify the error envelope, so this checks the shapes seen
/// in the wild: a `status: "fail"` marker and/or an `error` key holding either
/// an object or an array of objects with `error_code`/`error_message` fields.
fn extract_api_error(json: &Value) -> Option<(Option<i64>, String)> {
    let obj = json.as_object()?;

    let failed = obj.get("status").and_then(Value::as_str) == Some("fail");
    let error = obj.get("error");
    if !failed && error.is_none() {
        return None;
    }

    let first = match error {
        Some(Value::Array(items)) => items.first(),
        Some(v @ Value::Object(_)) => Some(v),
        _ => None,
    };

    if let Some(entry) = first {
        let code = entry.get("error_code").and_then(|c| {
            c.as_i64()
                .or_else(|| c.as_str().and_then(|s| s.parse().ok()))
        });
        let message = entry
            .get("error_message")
            .or_else(|| entry.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_string();
        return Some((code, message));
    }

    match error {
        Some(Value::String(s)) => Some((None, s.clone())),
        _ if failed => Some((None, "request failed".to_string())),
        _ => None,
    }
}

fn is_retryable(err: &SistrixError) -> bool {
    match err {
        SistrixError::Network(_) => true,
        SistrixError::Http { status, .. } => matches!(
            *status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        ),
        // In-band 429 (rate limit) and 500 (general/internal error) are transient.
        SistrixError::Api { code, .. } => matches!(*code, Some(429) | Some(500)),
        _ => false,
    }
}

fn preview(text: &str) -> &str {
    let end = text
        .char_indices()
        .nth(ERROR_BODY_PREVIEW)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_error_array() {
        let json = json!({
            "status": "fail",
            "error": [{"error_code": 100, "error_message": "wrong api key"}]
        });
        let (code, message) = extract_api_error(&json).unwrap();
        assert_eq!(code, Some(100));
        assert_eq!(message, "wrong api key");
    }

    #[test]
    fn extracts_error_object_with_string_code() {
        let json = json!({"error": {"error_code": "404", "error_message": "unknown method"}});
        let (code, message) = extract_api_error(&json).unwrap();
        assert_eq!(code, Some(404));
        assert_eq!(message, "unknown method");
    }

    #[test]
    fn plain_fail_status_is_an_error() {
        let json = json!({"status": "fail"});
        let (code, message) = extract_api_error(&json).unwrap();
        assert_eq!(code, None);
        assert_eq!(message, "request failed");
    }

    #[test]
    fn success_answers_pass_through() {
        for value in [
            json!({"answer": [{"credits": [{"value": 10000}]}], "credits": [{"used": 0}]}),
            json!([{"domain": "example.com"}]),
            json!({"status": "success"}),
        ] {
            assert!(extract_api_error(&value).is_none(), "false error: {value}");
        }
    }

    #[test]
    fn hints_cover_documented_codes() {
        for code in [
            100, 200, 403, 404, 429, 1000, 1001, 1002, 1004, 2000, 3000, 3502, 4000, 5000, 5001,
        ] {
            assert!(hint_for_code(code).is_some(), "missing hint for {code}");
        }
        assert!(hint_for_code(999).is_none());
    }

    #[test]
    fn retryable_classification() {
        assert!(is_retryable(&SistrixError::Network("timeout".into())));
        assert!(is_retryable(&SistrixError::Http {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: String::new(),
            hint: None,
        }));
        assert!(is_retryable(&SistrixError::Api {
            code: Some(429),
            message: "too many requests".into(),
            hint: None,
        }));
        assert!(is_retryable(&SistrixError::Api {
            code: Some(500),
            message: "internal error".into(),
            hint: None,
        }));
        assert!(!is_retryable(&SistrixError::Api {
            code: Some(100),
            message: "wrong api key".into(),
            hint: None,
        }));
        assert!(!is_retryable(&SistrixError::Http {
            status: StatusCode::UNAUTHORIZED,
            body: String::new(),
            hint: None,
        }));
    }
}
