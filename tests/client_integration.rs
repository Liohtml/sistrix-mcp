//! Offline integration tests for the SISTRIX HTTP client (wiremock).

use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use sistrix_mcp::client::{NetKind, SistrixClient, SistrixError};
use sistrix_mcp::config::{Args, Config};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const KEY: &str = "test-api-key-123";

async fn client_for(server: &MockServer) -> SistrixClient {
    SistrixClient::new(&Config::from_args(&sample_args(&server.uri())).unwrap()).unwrap()
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

#[tokio::test]
async fn response_timeout_is_not_retried_and_mentions_credits() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(5))
                .set_body_json(json!({"ok": true})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let mut args = sample_args(&server.uri());
    args.timeout_secs = 1;
    let client = SistrixClient::new(&Config::from_args(&args).unwrap()).unwrap();

    let err = client
        .call("domain.overview", &params(&[("domain", "example.com")]))
        .await
        .unwrap_err();
    let text = err.to_string();
    assert!(
        matches!(
            err,
            SistrixError::Network {
                kind: NetKind::Timeout,
                ..
            }
        ),
        "{text}"
    );
    assert!(
        text.contains("consumed API credits"),
        "error should mention possible credit usage: {text}"
    );

    // A mistaken retry would show up within the backoff window.
    tokio::time::sleep(Duration::from_millis(600)).await;
    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "a timed-out call must not be sent again (got {})",
        requests.len()
    );
}

#[tokio::test]
async fn connection_refused_is_retried() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let args = sample_args(&format!("http://{addr}/"));
    let client = SistrixClient::new(&Config::from_args(&args).unwrap()).unwrap();

    let start = Instant::now();
    let err = client.call("domain.overview", &[]).await.unwrap_err();
    let elapsed = start.elapsed();

    assert!(
        matches!(
            err,
            SistrixError::Network {
                kind: NetKind::Connect,
                ..
            }
        ),
        "{err}"
    );
    // Three attempts sleep 400ms + 800ms. One attempt returns immediately.
    assert!(
        elapsed >= Duration::from_millis(1_100),
        "connection refused was not retried (took {elapsed:?}): {err}"
    );
    assert!(
        !err.to_string().contains("consumed API credits"),
        "a refused connection was never billed: {err}"
    );
}

#[tokio::test]
async fn http_429_waits_for_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let start = Instant::now();
    let result = client.call("credits", &[]).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result, json!({"ok": true}));
    assert!(
        elapsed >= Duration::from_secs(1) && elapsed < Duration::from_secs(5),
        "expected the client to wait about 1s for Retry-After, took {elapsed:?}"
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

#[tokio::test]
async fn truncated_body_is_not_retried_and_mentions_credits() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_bg = Arc::clone(&hits);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_bg = Arc::clone(&stop);
    let background = std::thread::spawn(move || {
        while !stop_bg.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    hits_bg.fetch_add(1, Ordering::SeqCst);
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let mut buf = [0_u8; 4096];
                    let _ = stream.read(&mut buf);
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 64\r\nConnection: close\r\n\r\nnope",
                    );
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    let args = sample_args(&format!("http://{addr}/"));
    let client = SistrixClient::new(&Config::from_args(&args).unwrap()).unwrap();
    let err = client.call("domain.overview", &[]).await.unwrap_err();
    tokio::time::sleep(Duration::from_millis(700)).await;
    stop.store(true, Ordering::SeqCst);
    let _ = background.join();

    let text = err.to_string();
    assert!(
        matches!(
            err,
            SistrixError::Network {
                kind: NetKind::Body | NetKind::Timeout,
                ..
            }
        ),
        "{text}"
    );
    assert!(
        text.contains("consumed API credits"),
        "truncated body should mention possible credit usage: {text}"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "a truncated body was retried: {text}"
    );
}

fn sample_args(api_url: &str) -> Args {
    Args {
        api_key: Some(KEY.into()),
        country: None,
        api_url: api_url.into(),
        timeout_secs: 5,
        max_response_chars: 50_000,
        check: false,
    }
}
