//! Offline integration tests for the SISTRIX HTTP client (wiremock).

use serde_json::json;
use sistrix_mcp::client::{SistrixClient, SistrixError};
use sistrix_mcp::config::{Args, Config};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const KEY: &str = "test-api-key-123";

async fn client_for(server: &MockServer) -> SistrixClient {
    let args = Args {
        api_key: Some(KEY.into()),
        country: None,
        api_url: server.uri(),
        timeout_secs: 5,
        max_response_chars: 50_000,
        check: false,
    };
    let config = Config::from_args(&args).unwrap();
    SistrixClient::new(&config).unwrap()
}

fn params(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[tokio::test]
async fn posts_form_with_key_and_json_format() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/domain.overview"))
        .and(body_string_contains("api_key=test-api-key-123"))
        .and(body_string_contains("format=json"))
        .and(body_string_contains("domain=example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "answer": [{"sichtbarkeitsindex": [{"value": 42.7}]}],
            "credits": [{"used": 5}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let result = client
        .call("domain.overview", &params(&[("domain", "example.com")]))
        .await
        .unwrap();

    assert_eq!(
        result["answer"][0]["sichtbarkeitsindex"][0]["value"],
        json!(42.7)
    );
}

#[tokio::test]
async fn api_key_never_appears_in_the_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client.call("credits", &[]).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    for request in &requests {
        assert!(
            !request.url.as_str().contains(KEY),
            "api_key leaked into URL: {}",
            request.url
        );
    }
}

#[tokio::test]
async fn retries_transient_errors_then_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(2)
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let result = client.call("credits", &[]).await.unwrap();
    assert_eq!(result, json!({"ok": true}));
}

#[tokio::test]
async fn gives_up_after_max_attempts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(3)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.call("credits", &[]).await.unwrap_err();
    assert!(matches!(err, SistrixError::Http { status, .. } if status.as_u16() == 500));
}

#[tokio::test]
async fn client_errors_are_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.call("credits", &[]).await.unwrap_err();
    let text = err.to_string();
    assert!(text.contains("401"));
    assert!(text.contains("SISTRIX_API_KEY"), "missing hint: {text}");
}

#[tokio::test]
async fn in_band_errors_surface_code_message_and_hint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "fail",
            "error": [{"error_code": 200, "error_message": "not enough credits"}]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.call("domain.urls", &[]).await.unwrap_err();
    let text = err.to_string();
    assert!(text.contains("code 200"), "missing code: {text}");
    assert!(text.contains("not enough credits"));
    assert!(text.contains("sistrix_credits"), "missing hint: {text}");
}

#[tokio::test]
async fn api_key_is_scrubbed_from_error_messages() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "fail",
            "error": [{
                "error_code": 100,
                "error_message": format!("wrong api key: {KEY}")
            }]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.call("credits", &[]).await.unwrap_err();
    let text = err.to_string();
    assert!(!text.contains(KEY), "api_key leaked: {text}");
    assert!(text.contains("[REDACTED]"));
}

#[tokio::test]
async fn non_json_responses_are_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("<?xml version=\"1.0\"?><answer/>"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.call("credits", &[]).await.unwrap_err();
    assert!(matches!(err, SistrixError::InvalidResponse(_)));
}

#[tokio::test]
async fn consecutive_requests_are_spaced_out() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let start = std::time::Instant::now();
    client.call("credits", &[]).await.unwrap();
    client.call("credits", &[]).await.unwrap();

    assert!(
        start.elapsed() >= std::time::Duration::from_millis(300),
        "second request must wait for the 300 ms SISTRIX spacing, took {:?}",
        start.elapsed()
    );
}

#[tokio::test]
async fn method_becomes_the_request_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/keyword.seo.metrics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .call("keyword.seo.metrics", &params(&[("kw", "test")]))
        .await
        .unwrap();

    let requests: Vec<Request> = server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.path(), "/keyword.seo.metrics");
}
