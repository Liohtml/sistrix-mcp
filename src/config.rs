//! CLI arguments and runtime configuration.

use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use clap::Parser;
use url::{Host, Url};

/// Default SISTRIX API endpoint.
pub const DEFAULT_API_URL: &str = "https://api.sistrix.com/";

/// SISTRIX API key. `Debug` output is always `[REDACTED]`.
///
/// The raw value is available only through [`ApiKey::expose`], which the HTTP
/// client uses when building the form body and when scrubbing error text.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// The secret. Call this only at the request boundary.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

impl From<String> for ApiKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ApiKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl FromStr for ApiKey {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

/// MCP server for the SISTRIX SEO Toolbox.
#[derive(Parser, Debug)]
#[command(
    name = "sistrix-mcp",
    version,
    about = "MCP server for the SISTRIX SEO Toolbox — curated SEO tools over stdio",
    long_about = "Exposes a curated set of SISTRIX SEO tools to MCP clients \
                  (Claude, Cursor, VS Code, ...) plus a generic escape hatch for the full API.\n\n\
                  Minimal setup:\n\
                    sistrix-mcp --api-key YOUR_KEY"
)]
pub struct Args {
    /// SISTRIX API key. Create one under app.sistrix.com/account/api.
    /// If omitted, the server still starts and tool calls explain how to configure it.
    #[arg(short = 'k', long, env = "SISTRIX_API_KEY", hide_env_values = true)]
    pub api_key: Option<ApiKey>,

    /// Default country code (e.g. "de", "at", "fr") applied when a tool call
    /// does not pass one. Without it, SISTRIX uses the account's default country.
    #[arg(short = 'c', long, env = "SISTRIX_COUNTRY")]
    pub country: Option<String>,

    /// SISTRIX API base URL. Only change this for testing.
    #[arg(long, env = "SISTRIX_API_URL", default_value = DEFAULT_API_URL)]
    pub api_url: String,

    /// HTTP timeout per request, in seconds.
    #[arg(long, env = "SISTRIX_TIMEOUT_SECS", default_value_t = 30)]
    pub timeout_secs: u64,

    /// Maximum characters of a tool response before it is truncated (protects the LLM context).
    #[arg(long, env = "SISTRIX_MAX_RESPONSE_CHARS", default_value_t = 50_000)]
    pub max_response_chars: usize,

    /// Verify the API key against the SISTRIX API, print the remaining credits, then exit.
    #[arg(long)]
    pub check: bool,
}

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub api_url: Url,
    pub api_key: Option<ApiKey>,
    pub country: Option<String>,
    pub timeout_secs: u64,
    pub max_response_chars: usize,
}

impl Config {
    pub fn from_args(args: &Args) -> Result<Self> {
        let api_url = Url::parse(&args.api_url)
            .with_context(|| format!("invalid SISTRIX API URL: {}", args.api_url))?;
        if !matches!(api_url.scheme(), "http" | "https") {
            bail!("SISTRIX API URL must start with http:// or https://");
        }
        // The key travels in the POST body. Cleartext is only acceptable when
        // the bytes never leave the machine (wiremock and other local tests).
        if api_url.scheme() == "http" && !is_loopback_host(&api_url) {
            let host = api_url.host_str().unwrap_or("(no host)");
            bail!(
                "refusing cleartext SISTRIX API URL http://{host}: the API key would be sent \
                 without TLS. Use https://, or http://127.0.0.1 / http://localhost for local tests"
            );
        }

        let country = args
            .country
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_lowercase);
        if let Some(c) = &country {
            if !c.chars().all(|ch| ch.is_ascii_alphabetic()) || c.len() > 3 {
                bail!("invalid country code '{c}': expected a short code like 'de' or 'us'");
            }
        }

        Ok(Self {
            api_url,
            api_key: args
                .api_key
                .clone()
                .filter(|k| !k.expose().trim().is_empty()),
            country,
            timeout_secs: args.timeout_secs,
            max_response_chars: args.max_response_chars.max(1_000),
        })
    }
}

/// `http://` is allowed only for hosts that cannot leave the machine.
fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn base_args() -> Args {
        Args {
            api_key: Some("secret".into()),
            country: None,
            api_url: DEFAULT_API_URL.into(),
            timeout_secs: 30,
            max_response_chars: 50_000,
            check: false,
        }
    }

    #[test]
    fn accepts_defaults() {
        let cfg = Config::from_args(&base_args()).unwrap();
        assert_eq!(cfg.api_url.host_str(), Some("api.sistrix.com"));
        assert_eq!(cfg.api_key.as_ref().map(ApiKey::expose), Some("secret"));
    }

    #[test]
    fn debug_output_redacts_the_api_key() {
        let sentinel = "super-secret-sistrix-key-9f3c2a";
        let args = Args {
            api_key: Some(sentinel.into()),
            ..base_args()
        };
        let cfg = Config::from_args(&args).unwrap();
        let rendered = format!("{args:?}\n{cfg:?}\n{:?}", args.api_key);
        assert!(
            !rendered.contains(sentinel),
            "API key leaked via Debug: {rendered}"
        );
        assert!(rendered.contains("[REDACTED]"), "{rendered}");
        assert_eq!(cfg.api_key.as_ref().map(ApiKey::expose), Some(sentinel));
    }

    #[test]
    fn clap_debug_redacts_parsed_api_key() {
        let sentinel = "super-secret-sistrix-key-clap";
        let args = Args::try_parse_from(["sistrix-mcp", "--api-key", sentinel]).unwrap();
        let rendered = format!("{args:?}");
        assert!(!rendered.contains(sentinel), "{rendered}");
        assert_eq!(args.api_key.unwrap().expose(), sentinel);
    }

    #[test]
    fn rejects_non_http_scheme() {
        let mut args = base_args();
        args.api_url = "ftp://example.com".into();
        assert!(Config::from_args(&args).is_err());
    }

    #[test]
    fn rejects_cleartext_non_loopback_urls() {
        for url in [
            "http://api.sistrix.com/",
            "http://example.com/v1",
            "http://8.8.8.8/",
            "http://localhost.example.com/",
            "http://0.0.0.0/",
        ] {
            let mut args = base_args();
            args.api_url = url.into();
            let err = Config::from_args(&args).unwrap_err();
            let text = err.to_string();
            assert!(
                text.contains("without TLS"),
                "{url} should be refused, got: {text}"
            );
        }
    }

    #[test]
    fn allows_loopback_http_and_any_https() {
        for url in [
            "http://127.0.0.1:8080/",
            "http://127.0.0.1/",
            "http://localhost/",
            "http://LOCALHOST:9/path",
            "http://[::1]:8080/",
            "https://api.sistrix.com/",
            "https://example.com/http",
        ] {
            let mut args = base_args();
            args.api_url = url.into();
            Config::from_args(&args)
                .unwrap_or_else(|err| panic!("{url} should be accepted: {err}"));
        }
    }

    #[test]
    fn starts_without_api_key() {
        let mut args = base_args();
        args.api_key = None;
        let cfg = Config::from_args(&args).unwrap();
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn blank_api_key_counts_as_missing() {
        let mut args = base_args();
        args.api_key = Some("   ".into());
        let cfg = Config::from_args(&args).unwrap();
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn normalizes_country_code() {
        let mut args = base_args();
        args.country = Some(" DE ".into());
        let cfg = Config::from_args(&args).unwrap();
        assert_eq!(cfg.country.as_deref(), Some("de"));
    }

    #[test]
    fn rejects_bad_country_code() {
        let mut args = base_args();
        args.country = Some("deutschland".into());
        assert!(Config::from_args(&args).is_err());
    }

    #[test]
    fn enforces_minimum_response_budget() {
        let mut args = base_args();
        args.max_response_chars = 10;
        let cfg = Config::from_args(&args).unwrap();
        assert_eq!(cfg.max_response_chars, 1_000);
    }
}
