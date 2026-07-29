//! MCP server handler wiring the tool registry to the SISTRIX client.

use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData;
use tracing::debug;

use crate::client::SistrixClient;
use crate::tools::Registry;

/// Guidance returned when the server runs without a configured API key.
const UNCONFIGURED_HINT: &str =
    "SISTRIX is not configured. Set the SISTRIX_API_KEY environment variable — or the \
     --api-key flag — and restart the MCP server. Create a key under \
     https://app.sistrix.com/account/api. Example: SISTRIX_API_KEY=... sistrix-mcp";

#[derive(Clone)]
pub struct SistrixServer {
    client: Option<Arc<SistrixClient>>,
    registry: Arc<Registry>,
    max_response_chars: usize,
}

impl SistrixServer {
    pub fn new(
        client: Option<SistrixClient>,
        registry: Registry,
        max_response_chars: usize,
    ) -> Self {
        Self {
            client: client.map(Arc::new),
            registry: Arc::new(registry),
            max_response_chars,
        }
    }
}

/// Compact-serialize a response, truncating at a character budget so a single
/// tool call can never flood the model's context window.
pub fn shape_response(value: &serde_json::Value, max_chars: usize) -> String {
    let text = value.to_string();
    if text.chars().count() <= max_chars {
        return text;
    }
    let cut: String = text.chars().take(max_chars).collect();
    format!(
        "{cut}\n... [response truncated at {max_chars} characters — narrow the query with \
         a smaller 'limit' or more specific filters]"
    )
}

impl ServerHandler for SistrixServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            // rmcp negotiates down for older clients automatically.
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "sistrix-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("SISTRIX SEO Toolbox".to_string()),
                website_url: Some("https://github.com/Liohtml/sistrix-mcp".to_string()),
            },
            instructions: Some(format!(
                "SISTRIX SEO Toolbox. {count} tools.\n\n\
                 Workflow:\n\
                 1. Most SISTRIX calls cost weekly API credits (usually 1 per returned row); \
                 check the balance with sistrix_credits when in doubt and keep limits tight.\n\
                 2. Start domain questions with sistrix_domain_overview, then drill down \
                 (sistrix_visibility, sistrix_domain_rankings, sistrix_competitors, ...).\n\
                 3. Country indices matter: pass country='de'/'us'/... per call, discover codes \
                 via sistrix_lists.\n\
                 4. For anything not covered, call sistrix_api with any documented method \
                 (https://www.sistrix.com/api/).",
                count = self.registry.tool_count(),
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: self.registry.mcp_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.as_ref();
        let args = request.arguments.unwrap_or_default();
        debug!(tool = tool_name, "tool call");

        let invocation = match self.registry.resolve(tool_name, &args) {
            Ok(inv) => inv,
            Err(message) => return Ok(error_result(message)),
        };

        let Some(client) = &self.client else {
            return Ok(error_result(UNCONFIGURED_HINT.to_string()));
        };

        match client.call(&invocation.method, &invocation.params).await {
            Ok(value) => Ok(CallToolResult {
                content: vec![Content::text(shape_response(
                    &value,
                    self.max_response_chars,
                ))],
                is_error: Some(false),
                meta: None,
                structured_content: None,
            }),
            Err(err) => Ok(error_result(err.to_string())),
        }
    }
}

fn error_result(message: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(message)],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shape_passes_small_responses_through() {
        let value = json!({"credits": 10000});
        assert_eq!(shape_response(&value, 1_000), value.to_string());
    }

    #[test]
    fn shape_truncates_large_responses_with_guidance() {
        let value = json!(vec!["row"; 10_000]);
        let shaped = shape_response(&value, 500);
        assert!(shaped.contains("truncated at 500"));
        assert!(shaped.contains("limit"));
        assert!(shaped.chars().count() < 700);
    }

    #[test]
    fn shape_is_utf8_safe() {
        let value = json!("ü".repeat(2_000));
        let shaped = shape_response(&value, 100);
        assert!(shaped.contains("truncated"));
    }
}
