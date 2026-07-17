//! sistrix-mcp — MCP server for the SISTRIX SEO Toolbox.
//!
//! Exposes a curated set of SEO tools (plus a raw-API escape hatch)
//! to MCP clients over stdio. See `README.md` for usage.

pub mod client;
pub mod config;
pub mod server;
pub mod tools;
