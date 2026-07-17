# Entscheidung: Eigenbau in Rust statt Fork von ValentinPuma/sistrix-mcp

**Datum:** 2026-07-17 · **Entscheidung: Eigenbau** (Neuentwicklung in Rust nach dem matomo-mcp-Schema)

## Analyse des bestehenden Projekts

| Kriterium | Befund | Bewertung |
|---|---|---|
| Sprache | TypeScript (Node) | Wir wollen Rust: Single-Binary, Instant-Startup, kein Runtime |
| **Lizenz** | **Keine** (kein LICENSE-File, kein `license`-Feld) | **K.-o.-Kriterium:** ohne Lizenz gilt "all rights reserved" — Code darf weder geforkt noch wiederverwendet werden |
| Aktivität | Gesamtes Repo an einem Tag gepusht (2026-03-11), seither tot | Kein gepflegtes Upstream, auf das man aufbauen könnte |
| Community | 0 Stars, 0 Forks, 0 Issues | Kein Ökosystem-Verlust durch Neuanfang |
| Qualität | 82 Tools = fast 1:1-API-Spiegel; Cloud-Run/Docker-zentriert | Widerspricht unserem kuratierten Ansatz (Kontext-Budget des LLM) |
| Registry | Nicht auf npm veröffentlicht, kein server.json | Keine Distribution, die wir übernehmen würden |

## Was wir trotzdem daraus gelernt haben (nur Fakten, kein Code)

- Die 82-Tool-Liste diente als Coverage-Landkarte und deckt sich mit der offiziellen Doku
  (unabhängig verifiziert in [sistrix-api-reference.md](sistrix-api-reference.md)).
- Der `ai.check`-Bereich (5 Methoden) fehlt dort bereits — die API ist seit März 2026 gewachsen;
  ein 1:1-Spiegel veraltet schnell, ein Escape-Hatch nicht.

## Nachtrag: SISTRIX' eigener MCP-Server

SISTRIX dokumentiert unter `/api/connection-to-chatbot-ai/` eine eigene MCP-Anbindung
(gehostete Bridge, gleiche API-Keys, laut Doku zählen MCP-Anfragen derzeit **nicht** gegen die
Credit-Quote, Tool-Umfang nicht spezifiziert). Kein Grund gegen den Eigenbau — unser Server ist
selbst-gehostet, quelloffen, mit kuratierten Schemas und Escape-Hatch — aber die README
positioniert sich ehrlich dazu (Abschnitt „How is this different …").

## Konsequenz

Neuentwicklung `sistrix-mcp` in Rust, Schema wie [matomo-mcp](https://github.com/Liohtml/matomo-mcp):
kuratierte Tools statt API-Spiegel, `sistrix_api`-Escape-Hatch für alle 91+ Methoden,
API-Key-Scrubbing, Response-Budget, MIT-Lizenz, crates.io + GitHub Releases + Glama + Awesome-List.
