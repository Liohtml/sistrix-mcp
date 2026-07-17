# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/Liohtml/sistrix-mcp/releases/tag/v0.1.0
