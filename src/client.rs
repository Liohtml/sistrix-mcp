//! HTTP client for the SISTRIX API.
//!
//! All requests go through `POST https://api.sistrix.com/<method>` with
//! form-encoded parameters so the `api_key` never appears in URLs or server
//! access logs. Responses are requested as JSON (`format=json`). Failures
//! that are known not to have reached SISTRIX (connection refused, DNS) are
//! retried with exponential backoff. HTTP 429 honours `Retry-After`.
//! Timeouts and body-read failures are not retried: SISTRIX bills per
//! request, and a retry would charge the call again. SISTRIX allows 300
//! requests/minute with at least 300 ms between requests.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, warn};
use url::Url;

use crate::config::{ApiKey, Config};

const MAX_ATTEMPTS: u32 = 3;
const BACKOFF_BASE_MS: u64 = 400;
/// SISTRIX requires at least 300 ms between requests (300 requests/minute).
const MIN_REQUEST_SPACING: Duration = Duration::from_millis(300);
/// Don't sleep for a full minute because a `Retry-After` was huge.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(30);
/// Cap error bodies quoted back to the model.
const ERROR_BODY_PREVIEW: usize = 500;
/// Shared wording so every "maybe billed" error is recognizable.
const CREDIT_WARNING: &str =
    "The call may already have been processed and consumed API credits, so it was not retried";

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
        /// Server-supplied wait for HTTP 429, already capped at 30s.
        retry_after: Option<Duration>,
    },

    /// Transport failure. [`NetKind::Connect`] is safe to retry; the other
    /// kinds may already have been accepted (and billed) by SISTRIX.
    #[error("{message}")]
    Network { kind: NetKind, message: String },

    #[error("SISTRIX returned a non-JSON response: {0}")]
    InvalidResponse(String),
}

/// Why a request never produced a usable response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetKind {
    /// Nothing was sent (connection refused, DNS, unreachable host).
    Connect,
    /// Timed out after the request may already have been sent.
    Timeout,
    /// Response headers arrived, but the body could not be read.
    Body,
    /// Failed locally, before any bytes were sent.
    BeforeSend,
    /// Unclassified. Treated as possibly sent.
    Other,
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
    api_key: Option<ApiKey>,
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
            .map_err(|e| network_failure(NetKind::BeforeSend, self.scrub(&e.to_string())))?;

        let mut form: Vec<(&str, &str)> = vec![("format", "json")];
        if let Some(key) = &self.api_key {
            form.push(("api_key", key.expose()));
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
                    let delay = retry_delay(attempt, &err);
                    warn!(
                        method,
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        "retrying after transient error: {err}"
                    );
                    tokio::time::sleep(delay).await;
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
            .map_err(|e| network_failure(classify_send(&e), self.scrub(&e.to_string())))?;

        let status = response.status();
        // Read the header before `text()` consumes the response.
        let retry_after = (status == StatusCode::TOO_MANY_REQUESTS)
            .then(|| retry_after_from(response.headers()))
            .flatten();
        let text = response.text().await.map_err(|e| {
            let kind = if e.is_timeout() {
                NetKind::Timeout
            } else {
                NetKind::Body
            };
            network_failure(kind, self.scrub(&e.to_string()))
        })?;

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
                retry_after,
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
            Some(k) if !k.expose().is_empty() => text.replace(k.expose(), "[REDACTED]"),
            _ => text.to_string(),
        }
    }
}

fn network_failure(kind: NetKind, detail: String) -> SistrixError {
    let message = match kind {
        NetKind::Connect => format!(
            "could not reach the SISTRIX API: {detail}. Check your network and that api.sistrix.com is up."
        ),
        NetKind::BeforeSend => format!("could not build the SISTRIX request: {detail}."),
        NetKind::Timeout => format!("the SISTRIX request timed out: {detail}. {CREDIT_WARNING}."),
        NetKind::Body => {
            format!("the SISTRIX response could not be read: {detail}. {CREDIT_WARNING}.")
        }
        NetKind::Other => format!("the SISTRIX request failed: {detail}. {CREDIT_WARNING}."),
    };
    SistrixError::Network { kind, message }
}

/// Connection never established, so SISTRIX cannot have billed the call.
///
/// Checked before [`reqwest::Error::is_timeout`]: a connect timeout did not
/// leave the machine, whereas a response timeout may already have been billed.
fn classify_send(err: &reqwest::Error) -> NetKind {
    if err.is_builder() {
        return NetKind::BeforeSend;
    }
    if is_connect_failure(err) {
        return NetKind::Connect;
    }
    if err.is_timeout() {
        return NetKind::Timeout;
    }
    if err.is_body() || err.is_decode() {
        return NetKind::Body;
    }
    NetKind::Other
}

fn is_connect_failure(err: &reqwest::Error) -> bool {
    if err.is_connect() {
        return true;
    }
    let mut source = std::error::Error::source(err);
    while let Some(inner) = source {
        if let Some(io) = inner.downcast_ref::<std::io::Error>() {
            if matches!(
                io.kind(),
                std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::NotConnected
                    | std::io::ErrorKind::HostUnreachable
                    | std::io::ErrorKind::NetworkUnreachable
                    | std::io::ErrorKind::AddrNotAvailable
            ) {
                return true;
            }
        }
        source = inner.source();
    }
    false
}

/// How long to wait before the next attempt.
///
/// HTTP 429 uses `Retry-After` when SISTRIX sends one (delta-seconds or an
/// HTTP date), capped so a huge value cannot stall the MCP session. Anything
/// else uses exponential backoff.
fn retry_delay(attempt: u32, err: &SistrixError) -> Duration {
    if let SistrixError::Http {
        status: StatusCode::TOO_MANY_REQUESTS,
        retry_after: Some(after),
        ..
    } = err
    {
        return (*after).min(MAX_RETRY_AFTER);
    }
    let shift = attempt.saturating_sub(1).min(4);
    Duration::from_millis(BACKOFF_BASE_MS.saturating_mul(1_u64 << shift))
}

fn retry_after_from(headers: &HeaderMap) -> Option<Duration> {
    let raw = headers.get(RETRY_AFTER)?.to_str().ok()?;
    parse_retry_after(raw, SystemTime::now())
}

fn parse_retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.bytes().all(|b| b.is_ascii_digit()) {
        let secs: u64 = value.parse().ok()?;
        return Some(Duration::from_secs(secs).min(MAX_RETRY_AFTER));
    }
    let when = httpdate::parse_http_date(value).ok()?;
    Some(
        when.duration_since(now)
            .unwrap_or(Duration::ZERO)
            .min(MAX_RETRY_AFTER),
    )
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
        // Only when the request cannot have been billed. Timeouts and
        // truncated bodies are returned to the caller as-is.
        SistrixError::Network { kind, .. } => *kind == NetKind::Connect,
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
        assert!(is_retryable(&network(NetKind::Connect)));
        assert!(!is_retryable(&network(NetKind::Timeout)));
        assert!(!is_retryable(&network(NetKind::Body)));
        assert!(!is_retryable(&network(NetKind::BeforeSend)));
        assert!(!is_retryable(&network(NetKind::Other)));
        let timeout = network(NetKind::Timeout);
        assert!(
            timeout.to_string().contains("consumed API credits"),
            "{timeout}"
        );
        assert!(is_retryable(&SistrixError::Http {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: String::new(),
            hint: None,
            retry_after: None,
        }));
        assert!(is_retryable(&SistrixError::Http {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: String::new(),
            hint: None,
            retry_after: Some(Duration::from_secs(1)),
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
            retry_after: None,
        }));
    }

    #[test]
    fn retry_after_is_honoured_and_capped() {
        let now = httpdate::parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert_eq!(parse_retry_after("1", now), Some(Duration::from_secs(1)));
        assert_eq!(parse_retry_after("120", now), Some(Duration::from_secs(30)));
        assert_eq!(parse_retry_after("0", now), Some(Duration::ZERO));
        assert_eq!(
            parse_retry_after("Sun, 06 Nov 1994 08:49:39 GMT", now),
            Some(Duration::from_secs(2))
        );
        assert_eq!(
            parse_retry_after(
                "Sun, 06 Nov 1994 08:49:37 GMT",
                now + Duration::from_secs(5)
            ),
            Some(Duration::ZERO)
        );
        // A date more than 30s ahead is capped, not slept in full.
        assert_eq!(
            parse_retry_after("Sun, 06 Nov 1994 08:51:37 GMT", now),
            Some(Duration::from_secs(30))
        );
        assert_eq!(parse_retry_after("not-a-date", now), None);
        assert_eq!(parse_retry_after("", now), None);

        let with_header = SistrixError::Http {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: String::new(),
            hint: None,
            retry_after: Some(Duration::from_secs(1)),
        };
        assert_eq!(retry_delay(1, &with_header), Duration::from_secs(1));

        let uncapped = SistrixError::Http {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: String::new(),
            hint: None,
            retry_after: Some(Duration::from_secs(90)),
        };
        assert_eq!(retry_delay(1, &uncapped), MAX_RETRY_AFTER);

        let backoff = SistrixError::Http {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: String::new(),
            hint: None,
            retry_after: Some(Duration::from_secs(1)),
        };
        assert_eq!(retry_delay(1, &backoff), Duration::from_millis(400));
        assert_eq!(retry_delay(2, &backoff), Duration::from_millis(800));
    }

    #[test]
    fn debug_redacts_api_key_on_the_client() {
        let sentinel = "super-secret-sistrix-key-9f3c2a";
        let args = crate::config::Args {
            api_key: Some(sentinel.into()),
            country: None,
            api_url: crate::config::DEFAULT_API_URL.into(),
            timeout_secs: 30,
            max_response_chars: 50_000,
            check: false,
        };
        let cfg = Config::from_args(&args).unwrap();
        let client = SistrixClient::new(&cfg).unwrap();
        let rendered = format!("{args:?}\n{cfg:?}\n{client:?}");
        assert!(
            !rendered.contains(sentinel),
            "API key leaked via Debug: {rendered}"
        );
        assert!(rendered.contains("[REDACTED]"));
    }

    fn network(kind: NetKind) -> SistrixError {
        network_failure(kind, "detail".into())
    }
}
