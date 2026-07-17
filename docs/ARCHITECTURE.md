# Architecture

```
┌────────────┐  stdio/MCP   ┌──────────────────────────────┐  HTTPS POST  ┌────────────────┐
│ MCP client │ ───────────► │ sistrix-mcp                  │ ───────────► │ api.sistrix.com │
│ (Claude,…) │ ◄─────────── │  server → registry → client  │ ◄─────────── │                │
└────────────┘              └──────────────────────────────┘              └────────────────┘
```

## Modules

| Module | Responsibility |
|---|---|
| `config.rs` | CLI/env parsing (clap), validation, defaults |
| `tools.rs` | Declarative tool catalog + argument dispatch (no I/O) |
| `client.rs` | HTTP transport: form-encoded POST, retries, error mapping, key scrubbing |
| `server.rs` | MCP `ServerHandler`: list_tools / call_tool, response budget |
| `main.rs` | Wiring, logging (stderr), `--check` |

## Design decisions

**Curated catalog, not API mirroring.** The SISTRIX API has 91+ methods. Exposing
each as an MCP tool floods the model's context and measurably degrades tool
selection. Instead, `tools.rs` defines ~17 tools as `&'static` data
(`ToolSpec` / `SelectCase`): each maps a real SEO question to one or more API
methods via a `report`/`channel` select argument. `sistrix_api` remains as an
escape hatch, so nothing is unreachable — including future API areas.

**The `target`/`scope` model.** SISTRIX addresses its subject through one of
four query parameters (`domain`, `host`, `path`, `url` — the docs call this the
"address object"). Tools expose that as a required `target` string plus a
`scope` enum; `resolve_target` maps them onto the right parameter name and
rejects scopes a method doesn't support (e.g. backlink methods take no `url`).

**Credit awareness as a first-class concern.** SISTRIX bills per returned row
from a weekly budget. Defaults are deliberately tight (25 rows, not SISTRIX's
100–10,000), tool descriptions state the documented cost, and error code
200/3501 maps to a hint pointing at `sistrix_credits`.

**Per-case argument requirements.** Select cases can declare `requires`
(e.g. `project` for every `ai.tracker` report except the project list). The
dispatcher enforces them with errors that tell the model how to obtain the
missing value — the "call X first" pattern that makes agent loops converge.

**Key safety.** The `api_key` travels exclusively in POST bodies (never in
URLs → never in access logs) and is string-replaced with `[REDACTED]` in every
error path. The raw tool silently drops `api_key`/`format` overrides.

**Resilience.** 3 attempts with exponential backoff on 429/5xx/network errors
(SISTRIX allows 300 req/min). In-band errors (`{"status":"fail", "error":[…]}`)
are parsed defensively — the envelope is undocumented — and every documented
error code carries an actionable hint.

**Context budget.** `shape_response` hard-caps serialized responses
(default 50k chars, UTF-8-safe) and appends guidance on how to narrow the query.

## Testing

All tests run offline. Unit tests cover the dispatcher (schema generation,
target/scope mapping, case requirements, coercions) and error classification;
`tests/client_integration.rs` uses wiremock to verify transport behavior:
form encoding, retries, give-up, non-retry on 4xx, error envelope parsing,
key scrubbing, and that the key never appears in a URL.
