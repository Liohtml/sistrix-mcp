use anyhow::{Context, Result};
use clap::Parser;
use rmcp::{transport::stdio, ServiceExt};
use serde_json::Value;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use sistrix_mcp::client::SistrixClient;
use sistrix_mcp::config::{Args, Config};
use sistrix_mcp::server::SistrixServer;
use sistrix_mcp::tools::Registry;

#[tokio::main]
async fn main() -> Result<()> {
    // Logs go to stderr — stdout is reserved for the MCP stdio transport.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    let config = Config::from_args(&args)?;

    // Lazy configuration: the MCP server always starts and answers
    // introspection; without an API key, tool calls return setup guidance.
    let client = match &config.api_key {
        Some(_) => Some(SistrixClient::new(&config)?),
        None => {
            warn!("SISTRIX_API_KEY is not set — tool calls will return configuration guidance");
            None
        }
    };

    if args.check {
        let client =
            client.context("--check requires an API key (--api-key or SISTRIX_API_KEY)")?;
        return check(&client).await;
    }

    let registry = Registry::new(config.country.clone());
    info!(
        tools = registry.tool_count(),
        country = config.country.as_deref().unwrap_or("(account default)"),
        "starting sistrix-mcp on stdio"
    );

    let service = SistrixServer::new(client, registry, config.max_response_chars);

    let server = service
        .serve(stdio())
        .await
        .context("failed to start MCP server on stdio")?;
    server.waiting().await?;

    info!("sistrix-mcp stopped");
    Ok(())
}

/// `--check`: verify the API key and print the remaining credits, then exit.
async fn check(client: &SistrixClient) -> Result<()> {
    let response = client.call("credits", &[]).await?;
    match find_credits(&response) {
        Some(credits) => println!("✓ Connected — {credits} API credits available"),
        None => println!("✓ Connected — credits response: {response}"),
    }
    Ok(())
}

/// Pull the credit value out of the (loosely documented) response shape.
fn find_credits(value: &Value) -> Option<i64> {
    match value {
        Value::Object(obj) => {
            for (key, v) in obj {
                if key == "credits" || key == "value" {
                    if let Some(n) = v.as_i64() {
                        return Some(n);
                    }
                }
                if let Some(found) = find_credits(v) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(find_credits),
        _ => None,
    }
}
