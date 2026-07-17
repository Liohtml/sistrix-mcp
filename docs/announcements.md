# Announcement drafts

Ready-to-paste posts. Personal accounts required, so these are published
manually. Adjust tone freely — they're written to be honest, not salesy.

---

## 1. Reddit — r/rust

**Flair:** 🛠️ project
**Title:** `sistrix-mcp: an MCP server for the SISTRIX SEO Toolbox in Rust — credit-aware tool curation instead of API mirroring`

Repo: https://github.com/Liohtml/sistrix-mcp — MIT, `cargo install sistrix-mcp`

After matomo-mcp I wrapped a second analytics API for MCP, and SISTRIX added a
twist the first one didn't have: **every API call costs money-like credits**
(weekly budget, mostly 1 credit per returned row). That shaped the design:

**Credit-aware curation.** Instead of mirroring all 91 API methods as tools, it
exposes 17 hand-written tools whose defaults are deliberately tight (25 rows
instead of SISTRIX's 100–10,000), whose descriptions state the documented cost,
and whose error hints route the model to the free balance check before it burns
the week's budget. One `sistrix_api` escape hatch keeps the full API reachable.

**Rust bits that pulled their weight:**

- `rmcp` (official Rust MCP SDK) + tokio for the stdio transport
- the whole catalog is `&'static` data — a `cases!` macro builds the
  report→method dispatch tables, including per-case required arguments
  ("`project` is required for every tracker report except the list")
- wiremock for fully offline integration tests (retries, error envelopes,
  key scrubbing) — 46 tests, no SISTRIX account needed
- 6-target release matrix incl. musl static builds; ~4 MB stripped binary

**Safety posture:** the API key travels only in POST bodies (never URLs/logs),
is `[REDACTED]` in every error string, and reserved params can't be overridden
through tool arguments. Responses are hard-capped so one tool call can't flood
the model's context.

Happy to answer questions about rmcp, the dispatch design, or wrapping
credit-metered APIs for LLMs.

---

## 2. r/TechSEO / r/bigseo

**Title:** `sistrix-mcp — query SISTRIX from Claude/Cursor via MCP (open source)`

Open-sourced an MCP server for the SISTRIX API:
**https://github.com/Liohtml/sistrix-mcp**

With it connected, your AI assistant answers straight from SISTRIX data:

- *"SEO health check for example.com — visibility trend, competitors, biggest page-2 opportunities."*
- *"Does ChatGPT recommend our brand? Which sources does it cite?"* (the new AI-visibility endpoints)
- *"Search intent, CPC and SERP features for our top money keywords."*
- *"Amazon rankings and price history for this ASIN."*

Because the SISTRIX API bills per row from a weekly credit budget, the server
is deliberately credit-aware: tight default limits, cost notes on every tool,
free balance check. Single binary (Rust), MIT, works with Claude Desktop/Code,
Cursor, VS Code, Windsurf, Zed.

---

## 3. mcpservers.org submission form (wong2's list — no PRs accepted)

**URL:** https://mcpservers.org/submit

| Field | Value |
|---|---|
| Server Name | `sistrix-mcp` |
| Short Description | `Curated SISTRIX SEO Toolbox tools — visibility index, rankings, keyword research, backlinks, AI visibility (ChatGPT/Perplexity/AIO), Amazon marketplace — credit-aware defaults, full-API escape hatch. Rust, single binary.` |
| Link | `https://github.com/Liohtml/sistrix-mcp` |
| Category | Marketing / SEO (whichever the dropdown offers) |
| Contact Email | *(your private email)* |

The free tier is sufficient; the paid "premium" only buys faster review.

---

## 4. punkpeye/awesome-mcp-servers PR

Section: **Marketing** (SEO tools live there; alphabetical position). Entry format:

```markdown
- [Liohtml/sistrix-mcp](https://github.com/Liohtml/sistrix-mcp) [![Liohtml/sistrix-mcp MCP server](https://glama.ai/mcp/servers/Liohtml/sistrix-mcp/badges/score.svg)](https://glama.ai/mcp/servers/Liohtml/sistrix-mcp) 🦀 ☁️ 🍎 🪟 🐧 - Curated SISTRIX SEO Toolbox server — 17 credit-aware tools for visibility index, rankings, keyword research, backlinks, AI visibility (ChatGPT/Perplexity/AIO) and Amazon marketplace data, plus a full-API escape hatch. Single static binary with tight row limits and response budgets.
```
