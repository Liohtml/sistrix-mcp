# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-07-29

### Added

- Four guided MCP prompts (surfaced as one-click commands / slash-commands in
  clients): `seo_health_check`, `keyword_research`, `competitor_comparison`,
  and `ai_visibility_report`. Each walks the model through a proven,
  credit-aware analysis sequence — no tool knowledge required.
- Bulk keyword lookup: `sistrix_keyword` accepts an array of keywords for
  `report='metrics'` and `report='competition'` (SISTRIX's documented bulk
  format), so ten keywords cost one call instead of ten.
- Server-side regex filters: `regex_keyword`/`regex_url` on
  `sistrix_domain_rankings`, `regex_keyword` on `sistrix_keyword_ideas`
  (related searches), and `regex_url` on `sistrix_domain_structure`
  (top URLs) — only matching rows are returned and billed.
- Human-readable `title` and MCP tool annotations on every tool:
  `readOnlyHint: true` for all curated query tools, `readOnlyHint: false` +
  `destructiveHint: false` for the `sistrix_api` escape hatch (it can reach
  SISTRIX's additive write methods), `openWorldHint: true` throughout.
- Request pacing: consecutive API calls are spaced at least 300 ms apart
  (SISTRIX's documented minimum), shared across concurrent tool calls — so
  agentic bursts no longer trip the rate limit.

### Changed

- Declared MCP protocol version raised to 2025-03-26 (the version that
  introduced tool annotations); older clients are negotiated down
  automatically.

## [0.1.2] - 2026-07-23

### Fixed

- URL-shaped targets are normalized for the `domain`/`host` scopes: models
  frequently pass `https://example.com/` where SISTRIX expects a bare
  hostname — scheme, path, and query are now stripped automatically
  (`path`/`url` scopes stay untouched).
- Boolean flags passed as `false` are omitted from the API request entirely.
  Every documented SISTRIX flag defaults to `FALSE`, and presence-based
  parsing could otherwise flip a flag on.
- Whole-number floats (e.g. `25.0`) are accepted for integer arguments —
  some MCP clients serialize integers that way.
- Boolean string coercion is now case-insensitive (`"True"`, `"FALSE"`, ...).
- In-band SISTRIX error 500 (general/internal error) is retried with backoff
  like its HTTP counterpart.
- New actionable hints for error 1000 (*no results* — surfaced as an empty
  result, not a failure) and 1001 (invalid date).

### Added

- `glama.json` so the Glama MCP registry can verify the maintainer and
  re-index the repository.

### Changed

- Accurate credit notes: `sistrix_ai_entity` overview is a flat 10 credits.
- `sistrix_api` now warns that `project.create` / `project.start.onpage.check`
  write to the account and incur costs.
- README: honest comparison with SISTRIX's official hosted MCP bridge.

## [0.1.1] - 2026-07-17

### Fixed

- Declare the MCP Registry name (`mcp-name`) in the README so registry
  ownership validation of the crates.io package succeeds.

## [0.1.0] - 2026-07-17

Initial release.

### Added

- 17 curated MCP tools covering all eight SISTRIX API areas:
  domain, keyword, links, ai, ai.tracker, project (Optimizer), marketplace
  (Amazon), and the free basic functions (credits, countries, SERP features,
  AI models).
- `sistrix_api` escape hatch for every documented SISTRIX API method.
- `target` + `scope` model for SISTRIX's domain/host/path/url address objects.
- Credit-aware defaults: tight row limits and per-tool cost notes so a single
  question never burns the weekly credit budget.
- Optional default country (`SISTRIX_COUNTRY`) injected into every
  country-aware call.
- API key sent via POST body only, scrubbed from every error message.
- Automatic retries with exponential backoff on 429/5xx/network errors.
- Response truncation budget (`SISTRIX_MAX_RESPONSE_CHARS`) protecting the
  model's context window.
- `--check` command verifying the API key and printing the credit balance.
- Offline test suite (46 tests, wiremock-based, no SISTRIX account needed).
- CI (fmt, clippy, tests on Linux/macOS/Windows, MSRV 1.88, Docker build) and
  a 6-target release pipeline (incl. musl static builds) with crates.io,
  ghcr.io, and MCP Registry publishing.

[0.1.3]: https://github.com/Liohtml/sistrix-mcp/releases/tag/v0.1.3
[0.1.2]: https://github.com/Liohtml/sistrix-mcp/releases/tag/v0.1.2
[0.1.1]: https://github.com/Liohtml/sistrix-mcp/releases/tag/v0.1.1
[0.1.0]: https://github.com/Liohtml/sistrix-mcp/releases/tag/v0.1.0
