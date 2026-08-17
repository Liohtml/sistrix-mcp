<div align="center">

<img src="assets/logo.svg" width="120" alt="sistrix-mcp logo"/>

# sistrix-mcp

**Talk to your SISTRIX SEO data.** From Claude, Cursor, VS Code, or any MCP client.

[![CI](https://github.com/Liohtml/sistrix-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/Liohtml/sistrix-mcp/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/sistrix-mcp.svg)](https://crates.io/crates/sistrix-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-compatible-8A2BE2)](https://modelcontextprotocol.io)

*17 curated, credit-aware SEO tools + a full-API escape hatch. Single binary, instant startup, context-friendly.*

[Quickstart](#-quickstart) · [Clients](#-connect-your-client) · [Tools](#-tools) · [Credits](#-api-credits) · [Configuration](#%EF%B8%8F-configuration) · [FAQ](#-troubleshooting)

</div>

---

```text
You  ▸ How is example.com doing in Google — and who is beating us?

Claude ▸ example.com's visibility index is 4.82 (Germany), up 6% over the last
         8 weeks. It ranks for 18,400 organic keywords, 2,100 in the top 10.
         Closest SEO competitors: competitor-a.de (74% overlap),
         competitor-b.com (61%). Biggest opportunity: 312 keywords sit on
         page 2 — want the top 20 by potential gain?
```

Every question the SISTRIX Toolbox can answer, your AI assistant can now answer too — including follow-ups, comparisons, and "why?".

## ✨ Why sistrix-mcp?

| | |
|---|---|
| 🎯 **Curated, not generated** | 17 hand-crafted tools modeled on real SEO questions — not 90+ auto-generated API mirrors that flood the model's context and degrade tool selection. |
| 🧭 **Guided workflows** | Four MCP prompts (SEO health check, keyword research, competitor comparison, AI visibility report) run a complete credit-aware analysis with one click. |
| 💳 **Credit-aware** | SISTRIX bills most API calls per returned row from a weekly credit budget. Every tool ships tight row limits, cost notes in the descriptions, and a free `sistrix_credits` balance check — so one question never burns your week. |
| ⚡ **Instant startup** | No introspection round-trips. One static binary, no Node, no Python, no runtime. Starts in milliseconds. |
| 🔒 **Safe by default** | API key sent via POST body only (never in URLs/logs), redacted from every error message. |
| 🧠 **Context-friendly** | Row limits on every report and a hard response budget with actionable guidance — one tool call can never blow up the context window. |
| 🤖 **AI visibility included** | SISTRIX's newest data: which brands, entities, and sources ChatGPT, Perplexity, and Google AI Overviews actually cite (`sistrix_ai_*`). |
| 🛒 **Amazon included** | Marketplace data by ASIN: rankings, price history, reviews (`sistrix_amazon`). |
| 🧰 **Never a cage** | `sistrix_api` reaches **any** documented API method (ai.check, marketplace.keyword.*, project.onpage.*, …) when the curated tools don't cover it. |
| 🔁 **Resilient** | Automatic retries with backoff on 429/5xx/network hiccups. Every documented SISTRIX error code mapped to a hint the model can act on. |

## 🚀 Quickstart

### 1. Install

**Prebuilt binary** (Linux, macOS, Windows) — grab it from [Releases](https://github.com/Liohtml/sistrix-mcp/releases), or:

```bash
# Cargo
cargo install sistrix-mcp

# From source
cargo install --git https://github.com/Liohtml/sistrix-mcp

# Docker
docker pull ghcr.io/liohtml/sistrix-mcp
```

### 2. Get a SISTRIX API key

[app.sistrix.com/account/api](https://app.sistrix.com/account/api) → create a key.
API access is included from the **Plus** package onwards and uses a weekly credit budget.

### 3. Verify the connection

```bash
sistrix-mcp --api-key YOUR_KEY --check
```

```text
✓ Connected — 10000 API credits available
```

### 4. Connect your client ⬇

## 🔌 Connect your client

<details>
<summary><b>Claude Code</b></summary>

```bash
claude mcp add sistrix \
  --env SISTRIX_API_KEY=YOUR_KEY \
  --env SISTRIX_COUNTRY=de \
  -- sistrix-mcp
```

</details>

<details>
<summary><b>Claude Desktop</b></summary>

Add to `claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/`, Windows: `%APPDATA%\Claude\`):

```json
{
  "mcpServers": {
    "sistrix": {
      "command": "sistrix-mcp",
      "env": {
        "SISTRIX_API_KEY": "YOUR_KEY",
        "SISTRIX_COUNTRY": "de"
      }
    }
  }
}
```

</details>

<details>
<summary><b>Cursor</b></summary>

`.cursor/mcp.json` (project) or `~/.cursor/mcp.json` (global):

```json
{
  "mcpServers": {
    "sistrix": {
      "command": "sistrix-mcp",
      "env": {
        "SISTRIX_API_KEY": "YOUR_KEY",
        "SISTRIX_COUNTRY": "de"
      }
    }
  }
}
```

</details>

<details>
<summary><b>VS Code (GitHub Copilot)</b></summary>

`.vscode/mcp.json`:

```json
{
  "servers": {
    "sistrix": {
      "type": "stdio",
      "command": "sistrix-mcp",
      "env": {
        "SISTRIX_API_KEY": "${input:sistrix-key}",
        "SISTRIX_COUNTRY": "de"
      }
    }
  },
  "inputs": [
    {
      "id": "sistrix-key",
      "type": "promptString",
      "description": "SISTRIX API key",
      "password": true
    }
  ]
}
```

</details>

<details>
<summary><b>Windsurf / Zed / other MCP clients</b></summary>

Any client that speaks MCP over stdio works with the generic shape:

```json
{
  "command": "sistrix-mcp",
  "args": [],
  "env": {
    "SISTRIX_API_KEY": "YOUR_KEY",
    "SISTRIX_COUNTRY": "de"
  }
}
```

</details>

<details>
<summary><b>Docker (any client)</b></summary>

```json
{
  "mcpServers": {
    "sistrix": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "SISTRIX_API_KEY", "-e", "SISTRIX_COUNTRY",
        "ghcr.io/liohtml/sistrix-mcp"
      ],
      "env": {
        "SISTRIX_API_KEY": "YOUR_KEY",
        "SISTRIX_COUNTRY": "de"
      }
    }
  }
}
```

</details>

> [!TIP]
> Set `SISTRIX_COUNTRY` (e.g. `de`) and every tool call automatically uses that
> country index unless the model asks for another one.

## 🧭 Tools

| Tool | Answers questions like | Credits* |
|------|------------------------|----------|
| `sistrix_credits` | *"How many API credits do we have left?"* | free |
| `sistrix_lists` | *"Which country indices / SERP features / AI models exist?"* | free |
| `sistrix_domain_overview` | *"How is example.com doing overall?"* | 5 flat |
| `sistrix_visibility` | *"Visibility index trend over the last year? All-time high?"* | 1/row |
| `sistrix_keyword_counts` | *"How many keywords do we rank for? Top-10 trend?"* | 1/row |
| `sistrix_competitors` | *"Who are our closest SEO / Ads competitors?"* | 1/row |
| `sistrix_keyword_ideas` | *"Which keywords are just off page 1 with the biggest potential?"* | 1/row |
| `sistrix_domain_structure` | *"Which URLs/subdomains/directories drive our visibility?"* | 1/row |
| `sistrix_domain_rankings` | *"All keywords we rank for on page 2, with traffic."* | 1/row |
| `sistrix_keyword` | *"Volume, CPC, intent, SERP features for 'ergonomic chair'?"* | 1–5/row |
| `sistrix_links` | *"How does our backlink profile look? Top link texts?"* | 1/row–25 flat |
| `sistrix_ai_top` | *"Which brands does AI mention most in our market?"* | 1/row |
| `sistrix_ai_entity` | *"How do ChatGPT & Co. see our brand? Which sources do they cite?"* | 1/row–10 flat |
| `sistrix_ai_tracker` | *"How visible are we in our tracked AI prompts vs. competitors?"* | 1/row |
| `sistrix_project` | *"Optimizer project rankings, onpage crawl results, tracked SERPs."* | varies |
| `sistrix_amazon` | *"Amazon rankings, price and review history for this ASIN."* | 1/row |
| `sistrix_api` | Everything else — `ai.check.*`, `marketplace.keyword.*`, `project.onpage.*`, any documented method | varies |

<sub>*Costs as documented by SISTRIX per method; see [API credits](#-api-credits).</sub>

Domain tools accept a `target` (the domain, host, path, or URL) plus a `scope`
that tells SISTRIX how to interpret it, a `country` index, and a row `limit`.

### Guided workflows (MCP prompts)

Four ready-made analysis workflows ship as MCP prompts — clients like Claude
surface them as one-click commands. Each runs a proven, credit-aware tool
sequence and ends in a structured report:

| Prompt | Arguments | What you get |
|--------|-----------|--------------|
| `seo_health_check` | `target`, `country?` | Full status: key figures, trend, competitors, quick wins, next actions |
| `keyword_research` | `keyword`, `country?` | Metrics, intent, SERP features, questions, who ranks — plus a verdict |
| `competitor_comparison` | `target`, `competitor`, `country?` | Side-by-side comparison and 3 concrete moves to close the gap |
| `ai_visibility_report` | `brand`, `country?` | How ChatGPT & Co. see the brand: competitors, prompts, cited sources |

### Prompts to try

- *"Give me an SEO health check for example.com in Germany."*
- *"Which keywords sit on positions 11–20 with the most traffic potential?"*
- *"Compare our visibility trend with competitor-a.de over the last 12 months."*
- *"Does ChatGPT recommend our brand? Which sources does it cite in our niche?"*
- *"What's the search intent and CPC for our top 10 money keywords?"*

## 💳 API credits

SISTRIX API access starts with the **Plus** package and uses a **weekly** credit
budget. Most methods cost **1 credit per returned row**; a few are flat-rate
(`domain.overview` = 5, `domain.visibilityindex.overview` = 10,
`links.overview` = 25) and `keyword.seo.metrics` costs 5 per keyword.

sistrix-mcp defaults every list to **25 rows** (not SISTRIX's 100–10,000) and
tells the model what each call costs, so exploratory conversations stay cheap.
Check the balance anytime with `sistrix_credits` — it's free, as are
`sistrix_lists` and the project/tracker list reports.

The rate limit is 300 requests/minute; sistrix-mcp backs off and retries
automatically when it's hit.

## ⚙️ Configuration

| Flag | Env | Default | Description |
|------|-----|---------|-------------|
| `--api-key` | `SISTRIX_API_KEY` | — *(required)* | SISTRIX API key ([create here](https://app.sistrix.com/account/api)) |
| `--country` | `SISTRIX_COUNTRY` | account default | Default country index (e.g. `de`, `at`, `us`) for all country-aware tools |
| `--timeout-secs` | `SISTRIX_TIMEOUT_SECS` | `30` | Per-request timeout |
| `--max-response-chars` | `SISTRIX_MAX_RESPONSE_CHARS` | `50000` | Response budget before truncation |
| `--api-url` | `SISTRIX_API_URL` | `https://api.sistrix.com/` | API endpoint (testing only) |
| `--check` | — | — | Verify the API key, print remaining credits, then exit |

## 🆚 How is this different from SISTRIX's official MCP integration?

SISTRIX [documents its own MCP bridge](https://www.sistrix.com/api/connection-to-chatbot-ai/)
for connecting chatbots to the API — and per their docs, requests through it
currently don't count against the credit quota. If that covers your needs, use it!

sistrix-mcp is the open-source, self-hosted alternative: a local stdio binary
with a curated, fully documented tool surface (17 tools with exact schemas,
enums, and per-case validation), an escape hatch to **every** documented API
method including the newest areas (`ai.check`, marketplace, Optimizer onpage),
hard response budgets for the model's context, offline tests, and no dependency
on a hosted bridge. MIT-licensed, so you can read exactly what the model can and
cannot do with your API key.

## 🩺 Troubleshooting

<details>
<summary><b>"wrong api key" (code 100)</b></summary>

Run `sistrix-mcp --api-key ... --check`. If it fails, regenerate the key under
[app.sistrix.com/account/api](https://app.sistrix.com/account/api).

</details>

<details>
<summary><b>"not enough credits" (code 200)</b></summary>

The weekly budget is exhausted. Credits refill weekly; check the balance with
`sistrix_credits`. Keep `limit` small — most tools cost 1 credit per row.

</details>

<details>
<summary><b>"package does not contain api access" (code 5000)</b></summary>

API access starts with the SISTRIX **Plus** package. Check your plan.

</details>

<details>
<summary><b>"domain not found" (code 2000)</b></summary>

The domain isn't in the SISTRIX database for the chosen country index. Try
another `country`, or check the spelling.

</details>

<details>
<summary><b>Responses feel truncated</b></summary>

That's the context guard doing its job. Ask for fewer rows or raise
`--max-response-chars`.

</details>

## 🏗️ Development

```bash
git clone https://github.com/Liohtml/sistrix-mcp.git
cd sistrix-mcp
cargo test                                  # 59 offline tests, no SISTRIX account needed
cargo clippy --all-targets -- -D warnings
cargo run -- --api-key YOUR_KEY --check
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the design, and
[docs/sistrix-api-reference.md](docs/sistrix-api-reference.md) for a complete,
crawled reference of the SISTRIX API (all 91 methods with parameters and
credit costs).

Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

[MIT](LICENSE) © Lionel Machire

- MCP Registry name: `mcp-name: io.github.Liohtml/sistrix-mcp`
- Also listed on Glama:

<a href="https://glama.ai/mcp/servers/Liohtml/sistrix-mcp">
  <img width="380" height="200" src="https://glama.ai/mcp/servers/Liohtml/sistrix-mcp/badge" alt="sistrix-mcp on Glama" />
</a>

*Not affiliated with or endorsed by SISTRIX GmbH. SISTRIX is a trademark of SISTRIX GmbH.*
