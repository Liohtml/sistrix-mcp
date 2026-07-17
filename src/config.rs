//! CLI arguments and runtime configuration.

use anyhow::{bail, Context, Result};
use clap::Parser;
use url::Url;

/// Default SISTRIX API endpoint.
pub const DEFAULT_API_URL: &str = "https://api.sistrix.com/";

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
    pub api_key: Option<String>,

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
    pub api_key: Option<String>,
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
            api_key: args.api_key.clone().filter(|k| !k.trim().is_empty()),
            country,
            timeout_secs: args.timeout_secs,
            max_response_chars: args.max_response_chars.max(1_000),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(cfg.api_key.as_deref(), Some("secret"));
    }

    #[test]
    fn rejects_non_http_scheme() {
        let mut args = base_args();
        args.api_url = "ftp://example.com".into();
        assert!(Config::from_args(&args).is_err());
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
