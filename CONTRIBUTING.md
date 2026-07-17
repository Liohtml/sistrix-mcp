# Contributing to sistrix-mcp

Thanks for considering a contribution! This project aims to stay small, fast, and
useful — every PR that moves it in that direction is welcome.

## Development setup

```bash
git clone https://github.com/Liohtml/sistrix-mcp.git
cd sistrix-mcp
cargo test          # unit + integration tests (no SISTRIX instance needed)
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

The integration tests run against an in-process mock server (wiremock), so the
full suite works offline. To test against the real API (needs a key with API access):

```bash
cargo run -- --api-key YOUR_KEY --check
```

## Guidelines

- **Keep the tool catalog curated.** New tools should answer a question a real
  user asks ("which keywords sit just off page 1?"), not mirror one API method 1:1.
  Anything exotic is already reachable via `sistrix_api`.
- **Tool descriptions are UX.** They are the only thing the model sees — write
  them for an LLM choosing between 17 tools.
- **Every behavior change needs a test.** `cargo test` must stay green on
  Linux, macOS, and Windows.
- **No new dependencies without a reason.** Startup time and binary size are
  features.

## Commit / PR conventions

- Conventional commits appreciated (`feat:`, `fix:`, `docs:`, ...), not enforced.
- One logical change per PR keeps reviews fast.
- CI (fmt, clippy, tests on 3 platforms, MSRV, Docker build) must pass.

## Reporting bugs

Open an issue with your SISTRIX package, how the server is launched
(binary/Docker), and the exact error output — API key redacted, obviously.
