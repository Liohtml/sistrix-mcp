# SISTRIX API — Referenz

> Quelle: offizielle SISTRIX-API-Dokumentation unter `https://www.sistrix.com/api/`.
> Alle Angaben stammen ausschließlich aus den gecrawlten Seiten. Wo die Doku etwas nicht angibt,
> steht ausdrücklich **not stated** — es wurde nichts ergänzt oder geraten.
>
> **Stand:** 17.07.2026. 104 Seiten gecrawlt, **keine einzige unerreichbar**. 91 Methoden in
> 8 Bereichen. Siehe [Crawl-Log](#crawl-log).

## Übersicht

Die SISTRIX-API stellt SEO-, Keyword-, Backlink-, AI- und Marketplace-Daten über HTTP bereit.
Methoden sind nach Bereichen benannt (`domain.*`, `keyword.*`, `links.*`, `ai.*`, `project.*`,
`marketplace.*`) und werden als Pfad direkt an die Basis-URL gehängt.

**Basis-URL:** `https://api.sistrix.com/`

Eine Methode wird als Pfadsegment angehängt, z. B.:

```
https://api.sistrix.com/domain.overview?api_key=...&domain=example.com&format=json
```

Die Doku zeigt für die Methodenseiten POST-Requests; Query-Parameter funktionieren ebenfalls
(das Getting-Started-Beispiel nutzt GET mit Query-String).

## Authentifizierung

| Aspekt | Wert |
|---|---|
| Schema | API-Key als Parameter `api_key` (Query-String oder POST-Body) |
| Parametername | `api_key` (STRING, immer Pflicht) |
| Key beziehen | Account-Tab „API Access": `https://app.sistrix.com/account/api` |
| HTTP-Header | **Nicht dokumentiert** — die Doku nennt ausschließlich die Parameter-Übergabe |

Zitat: „This key identifies your account and manages credit usage, so it must **not be shared**."

Beispiel:

```
https://api.sistrix.com/credits?api_key=Hb...MJ&format=json
```

## Format-Parameter

| Parameter | Typ | Pflicht | Werte | Default |
|---|---|---|---|---|
| `format` | STRING | optional | `json`, `xml` | `xml` |

Zitat: „The SISTRIX API offers two response modes, selectable via the `format` parameter. You can
receive the response as a **JSON** array, or alternatively, choose an **XML** format."
JSONP wird **nicht** erwähnt.

> **Hinweis zu Beispiel-Responses:** SISTRIX rendert Beispieldaten hinter einem Control
> „Show example data", das einen API-Key voraussetzt. Deshalb liefert **keine** der gecrawlten
> Seiten einen literalen Response-Body. Dokumentiert sind daher die von SISTRIX beschriebenen
> Response-Felder, keine erfundenen Payloads.

## Credit-System

| Aspekt | Angabe |
|---|---|
| Zuteilung | Wöchentliches Kontingent („a weekly allocation of credits") |
| Ab Paket | Plus, Professional, Premium |
| Kosten | Stehen in der Doku neben dem Funktionsnamen. „If no price is shown, the function is free to use." |
| Kostenlogik | „Typically, the cost is based on the number of data points retrieved." |
| Stand abfragen | Methode `credits` oder Account-Tab „API Access" |
| Verbrauch pro Call | „At the end of each API response you can find the amount of credits used." |

Beobachtete Kostenmuster: die meisten Methoden kosten **1 Credit pro zurückgegebenem Eintrag**;
Ausnahmen sind Pauschalen (`domain.overview` = 5, `domain.visibilityindex.overview` = 10,
`links.overview` = 25), erhöhte Sätze (`keyword.seo.metrics` = 5/Eintrag) und Deckel
(`links.list` = 1/Eintrag, max. 250 pro Query).

## Rate Limits

| Aspekt | Angabe |
|---|---|
| Limit | Max. **300 Requests pro Minute** pro Toolbox-Account |
| Mindestabstand | **300 ms** zwischen Requests |
| Überschreitung | HTTP **429** (Too Many Requests) |
| Limit prüfen | `credits`-Funktion mit cURL-Parameter `-I`; Header `X-RateLimit-Limit` (gesamt) und `X-RateLimit-Remaining` (verfügbar) |

Die Nutzung wird aktiv überwacht: „Violations can lead to temporary or permanent suspension of API
access." Untersagt sind das Umgehen von Rate Limits, das Testen von Sicherheitslücken und
absichtliches Überlasten der API.

**Datenweitergabe:** „data, functions, and results provided by the API may not be shared with third
parties" ohne schriftliche Genehmigung.

## Fehlercodes

Das Fehler-Response-Format (XML/JSON-Struktur) wird von der Doku **nicht** angegeben.

| Code | Message | Bedeutung |
|---|---|---|
| 100 | wrong api key | An incorrect/invalid API key was entered. |
| 200 | not enough credits | There are not enough credits left on the API credit account. |
| 403 | permission denied | No access permission. |
| 404 | unknown method | The method entered is unknown. |
| 429 | too many requests | Too many queries were made in one minute. |
| 500 | internal error | General error. |
| 1000 | no results | The query returns no results. |
| 1001 | no valid date | No valid date was entered. |
| 1002 | invalid parameters | An invalid parameter was used. |
| 1003 | missing parameters | A required parameter is missing. |
| 1004 | function no longer supported | This function is no longer available. |
| 2000 | domain not found | The specified domain could not be found. |
| 2001 | domain not monitored | The specified domain is not monitored. |
| 3000 | keyword not found | The specified keyword could not be found. |
| 3001 | no keyword history | This keyword is in the extended keyword database. SERP archive unavailable. |
| 3501 | not enough credits left to fetch the given bulk | Insufficient API credits for query execution. |
| 3502 | too many parameters | More parameters used than allowed by this query. |
| 4000 | project not found | The specified project could not be found. |
| 4001 | no current projects | No projects could be found. |
| 4002 | no keywords projects | No keywords could be found for the project. |
| 4003 | no keyword found | No keyword could be found. |
| 4004 | no valid issue | No valid value was entered. |
| 4005 | invalid domain | The domain deposited is invalid. |
| 4006 | cannot recrawl | The project cannot be crawled again. |
| 4007 | missing parameters | A required parameter is missing. |
| 5000 | package does not contain api access | No API access via booked package. |
| 5001 | action requires unlimited api access | This request requires unlimited API access. |

## Länder-Indizes

Die unterstützten Länder werden **nicht** statisch in der Doku aufgelistet. Sie sind zur Laufzeit
über die Methode `countries` abrufbar, die „a list of all available countries for the Google
functions and the marketplace functions" liefert; jeder Eintrag enthält Ländercode und Ländername.

Fast alle Methoden akzeptieren einen optionalen Parameter `country` (STRING/countrycode). Ohne
Angabe gilt das Standardland des Toolbox-Accounts.

---

# Basisfunktionen

Nicht einem Bereich zugeordnet, „designed to assist with the overall operation of the SISTRIX-API".
Für alle drei ist die Credit-Angabe **not stated** (laut Getting-Started-Regel „If no price is shown,
the function is free to use").

### countries
**Endpoint:** `https://api.sistrix.com/countries` (POST)
**Credits:** not stated
**Description:** Liefert alle für die Google- und Marketplace-Funktionen verfügbaren Länder.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key zur Authentifizierung |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Liste aller verfügbaren Länder; jeder Eintrag enthält Ländercode und Ländername.
Kein literales Beispiel auf der Seite.

### credits
**Endpoint:** `https://api.sistrix.com/credits` (POST)
**Credits:** not stated
**Description:** „Returns the available credits for the specified API key."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key zur Authentifizierung |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Enthält die verbleibenden API-Credits des Accounts. Kein literales Beispiel.
**Notes:** Mit cURL-Parameter `-I` liefert diese Funktion die Rate-Limit-Header
`X-RateLimit-Limit` und `X-RateLimit-Remaining`.

### serpfeatures
**Endpoint:** `https://api.sistrix.com/serpfeatures`
**Credits:** not stated
**Description:** „Returns all available SERP Features for filter used in the API."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key zur Authentifizierung |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Liste der verfügbaren SERP-Features. Die Doku **enumeriert die Features nicht** —
sie sind zur Laufzeit über diese Methode abzurufen.

---

# domain

16 Methoden. Laut Übersichtsseite: „Creates a list of available domain methods for the given DOMAIN,
HOSTNAME, URL, or PATH." Bereichsweit Pflicht: `api_key` und ein `address_object`; `format`
(XML/JSON) ist durchgängig optional. Die Übersichtsseite selbst nennt keine Credit-Kosten pro Methode.

### domain.competitors.sem
**Endpoint:** `https://api.sistrix.com/domain.competitors.sem`
**Credits:** 1 / zurückgegebenem Eintrag
**Description:** Liste der ADs-Wettbewerber und ihres Ähnlichkeitsgrads (%) zum gegebenen DOMAIN, PATH oder URL.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING (domain/host/path/url) | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Land für den Vergleich. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Wettbewerber. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Wettbewerber nach Ähnlichkeit sortiert; das angefragte Objekt erscheint als 100-%-Treffer.
**Notes:** Codebeispiele in PHP, Python, cURL, Node.js.

### domain.competitors.seo
**Endpoint:** `https://api.sistrix.com/domain.competitors.seo`
**Credits:** 1 / Eintrag
**Description:** „a list of SEO competitors and their degree of similarity (%) with the given DOMAIN, PATH, or URL."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING (domain/host/path/url) | Objekt für den Vergleich |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Wettbewerber. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Wettbewerber nach Ähnlichkeit sortiert, angefragtes Objekt bei 100 %.

### domain.hosts
**Endpoint:** `https://api.sistrix.com/domain.hosts`
**Credits:** 1 / Eintrag
**Description:** „the top 100 ranking hosts for the given DOMAIN".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain) | ja | STRING | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Hosts. Default: 100 |
| offset | nein | INTEGER | Start-Rang. Default: 0 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Felder `host`, `top10` (Anzahl Top-10-Keywords), `top100` (Anzahl Top-100-Keywords),
plus prozentualer Anteil am Gesamt-Sichtbarkeitsindex je Host.
**Notes:** `limit` + `offset` ermöglichen Pagination.

### domain.ideas
**Endpoint:** `https://api.sistrix.com/domain.ideas`
**Credits:** 1 / Eintrag
**Description:** Keyword-Vorschläge „based on Google's related search queries shown at the end of search result pages".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Land der Vorschläge. Default: Account-Standard |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Seitennummer. Default: not stated |
| regex_keyword | nein | STRING | Regex-Filter auf Keywords. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Eintrag: Keyword, Domain (potenziell rankende Domain), Competition-Level.
**Notes:** Regex-Filter greifen nur auf das Keyword-Feld. Regex-Sonderzeichen müssen URL-kodiert
werden (z. B. `+` als `%2B`).

### domain.kwcount.sem
**Endpoint:** `https://api.sistrix.com/domain.kwcount.sem`
**Credits:** 1 / Eintrag
**Description:** „Returns the number of addwords for the given DOMAIN, HOST, URL, or PATH."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING (domain/host/path/url) | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Begrenzt die Anzahl historischer Werte (mit `history`). Default: 20 |
| history | nein | BOOLEAN | TRUE = historische wöchentliche Addword-Counts. Default: FALSE |
| date | nein | DATE | Bestimmtes Datum für den Addword-Count. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Anzahl gezählter Addwords (`value`), Stichtag (`date`), Domain des Fundes.
**Notes:** Historische Daten umfassen per Default bis zu 20 Wochen.

### domain.kwcount.seo
**Endpoint:** `https://api.sistrix.com/domain.kwcount.seo`
**Credits:** 1 / Eintrag
**Description:** Anzahl organischer Keywords für DOMAIN, HOST, URL oder PATH.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING (domain/host/path/url) | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Standardland des Toolbox-Accounts |
| limit | nein | INTEGER | Begrenzt historische Werte. Default: 100 (bei `history=TRUE`) |
| history | nein | BOOLEAN | TRUE = historische wöchentliche Keyword-Counts. Default: FALSE |
| date | nein | DATE | Bestimmtes Datum. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „the number of counted keywords (value), the date for which this count applies (date),
and the domain where the keywords were found."
**Notes:** Mit `history` per Default „the last 100 weeks".

### domain.kwcount.seo.top10
**Endpoint:** `https://api.sistrix.com/domain.kwcount.seo.top10`
**Credits:** 1 / Eintrag
**Description:** „Returns the amount of TOP10 keywords found in the organic Google index for the given DOMAIN, HOST, PATH, or URL."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Domain, Host, Path oder URL |
| date | nein | DATE | Stichtag für den TOP10-Count. Default: aktuelles Datum |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| history | nein | BOOLEAN | TRUE = historische wöchentliche TOP10-Counts. Default: FALSE |
| extended | nein | BOOLEAN | Liefert erweiterte Daten für historische und aktuelle Datensätze. Default: not stated |
| limit | nein | INTEGER | Begrenzt Counts bei `history=TRUE`. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Keyword-Count-`value`, Stichtag, Domain.

### domain.opportunities
**Endpoint:** `https://api.sistrix.com/domain.opportunities`
**Credits:** 1 / Eintrag
**Description:** Findet Keywords, für die eine Domain aktuell nicht auf Seite 1 rankt, diese Position
aber durch Optimierung erreichen könnte.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Keyword-Opportunities. Default: 100 |
| offset | nein | INTEGER | Startpunkt für Pagination. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Enthält eine `gain`-Metrik (Skala 0–100) für das Optimierungspotenzial.

### domain.overview
**Endpoint:** `https://api.sistrix.com/domain.overview`
**Credits:** **5** (Pauschale)
**Description:** „Creates an overview of the most recent relevant key figures of the given DOMAIN, HOSTNAME, URL, or PATH."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Domain, Hostname, Path oder URL |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Sichtbarkeitsindex (`sichtbarkeitsindex`), SEO-Keyword-Count (`kwcount.seo`) und
AdWords-Count (`kwcount.sem`), jeweils mit Datum und Wert.

### domain.paths
**Endpoint:** `https://api.sistrix.com/domain.paths`
**Credits:** 1 / Eintrag
**Description:** „top 100 ranking paths for the given DOMAIN, HOST".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain/host) | ja | STRING | Domain oder Host |
| country | nein | STRING/countrycode | Land der Analyse. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Pfade. Default: 100 |
| offset | nein | INTEGER | Start-Rangposition. Default: 0 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Pfadname, `top10`, `top100` und prozentualer Anteil am Gesamt-Sichtbarkeitsindex je Pfad.
**Notes:** Abweichend von den meisten domain-Methoden ist das `address_object` hier nur als
Domain oder Host dokumentiert.

### domain.ranking.distribution
**Endpoint:** `https://api.sistrix.com/domain.ranking.distribution`
**Credits:** 1 / Eintrag
**Description:** Verteilt Keywords auf Google-Ergebnisseiten (Seite 1 = Position 1–10, Seite 2 = 11–20 usw.).

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Geografischer Fokus. Default: Account-Standard |
| history | nein | BOOLEAN | Historische Verteilungen; ohne `limit` die letzten 10 Werte. Default: false |
| percent | nein | BOOLEAN | Prozentwerte statt absoluter Zahlen. Default: false |
| date | nein | DATE | Verteilung für die angegebene Woche. Default: aktuelle Werte |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** „an array representing the ranking distribution for the requested date(s). Each entry
includes the date (date) … and the number of ranking keywords for each of the first 10 result pages
(page1, page2, etc.)."

### domain.traffic.estimation
**Endpoint:** `https://api.sistrix.com/domain.traffic.estimation`
**Credits:** 1 / Eintrag
**Description:** „provides the top 100 paths with the highest traffic for the given DOMAIN, HOST, PATH, or URL."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Zu analysierendes Objekt |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Anzahl Ergebnisse. Default: 100 |
| offset | nein | INTEGER | Start-Rangposition. Default: 0 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Pfad: geschätzte Klicks, prozentualer Klick-Anteil, geschätzter Domain-Wert,
Anzahl Top-100-Keywords.

### domain.urlcount.seo
**Endpoint:** `https://api.sistrix.com/domain.urlcount.seo`
**Credits:** 1 / Eintrag
**Description:** „Provides the top 10 and top 100 URL counts for the given DOMAIN, HOST, PATH, or URL."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING (domain/host/path/url) | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Anzahl Ergebnisse. Default: 100 |
| date | nein | DATE | Stichtag für den URL-Count. Default: not stated |
| history | nein | BOOLEAN | TRUE = historische wöchentliche URL-Counts. Default: FALSE |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „Each entry includes the date of the evaluation (date), the number of URLs ranking in
the top 10 (top10), and the number of URLs ranking in the top 100 (top100)."

### domain.urls
**Endpoint:** `https://api.sistrix.com/domain.urls`
**Credits:** 1 / Eintrag
**Description:** Zeigt per Default die Top-100-Ranking-URLs des Objekts.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Abzufragendes Objekt |
| country | nein | STRING/countrycode | Ländercode. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl URLs. Default: 100 |
| offset | nein | INTEGER | Start-Rangposition. Default: not stated |
| date | nein | DATE | Stichtag. Default: not stated |
| regex_url | nein | STRING | Regex-Filter auf URLs. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** „a list of URLs (url), including the count of top 10 ranking keywords (top10), the
count of top 100 ranking keywords (top100), and the percentage of the overall visibility index."
**Notes:** Regex-Parameter müssen URL-kodiert werden (`+` → `%2B`). POST wird unterstützt.

### domain.visibilityindex
**Endpoint:** `https://api.sistrix.com/domain.visibilityindex` (POST)
**Credits:** 1 / Eintrag
**Description:** Sichtbarkeitsindex für das Objekt, aktuell oder historisch.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| history | nein | BOOLEAN | TRUE = alle verfügbaren historischen Wochenwerte. Default: FALSE |
| limit | nein | INTEGER | Begrenzt historische Werte. Default: not stated |
| daily | nein | BOOLEAN | TRUE = täglicher Sichtbarkeitsindex der letzten 30 Tage. Default: FALSE |
| date | nein | DATE | Stichtag für den Wochenwert. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „a JSON or XML array with the visibility index (value) for the supplied domain, along
with the domain name (domain) and the date (date) of the measurement."
**Notes:** Historie über `history`, `daily`, `limit`, `date` kombinierbar. `history=TRUE` ohne
`limit` liefert alle verfügbaren historischen Werte.

### domain.visibilityindex.overview
**Endpoint:** `https://api.sistrix.com/domain.visibilityindex.overview` (POST)
**Credits:** **10** (Pauschale)
**Description:** Höchster und niedrigster Sichtbarkeitsindex für Domain, Hostname, URL oder Path.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| daily | nein | BOOLEAN | TRUE = höchster/niedrigster täglicher Sichtbarkeitsindex. Default: false |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `sichtbarkeitsindex_overview_max` und `sichtbarkeitsindex_overview_min`, je mit Wert
und Datum; zusätzlich der abgedeckte Zeitraum in `sichtbarkeitsindex_overview` über den Parameter
`weeks`.
**Notes:** `daily` ermöglicht laut Seite eine 100-Tage-Peak-Analyse.

---

# keyword

11 Methoden plus Übersichtsseite.

### keyword (Übersicht)
**Endpoint:** `https://api.sistrix.com/keyword`
**Credits:** not stated
**Description:** Liefert eine Liste aller auf das Keyword anwendbaren keyword-Methoden.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** JSON-/XML-Array mit allen verfügbaren keyword-Methoden (Name + Funktions-URL).
**Notes:** POST.

### keyword.domain.sem
**Endpoint:** `https://api.sistrix.com/keyword.domain.sem`
**Credits:** 1 / Eintrag
**Description:** „Displays the AdWords rankings, including position, competition, and traffic information for the specified DOMAIN, HOST, PATH, or URL."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Abzufragendes Objekt |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| date | nein | DATE | Woche, für die Keyword-Daten geholt werden |
| limit | nein | INTEGER | Max. Anzahl AdWords. Default: 10.000 |
| offset | nein | INTEGER | Startposition (überspringt Keywords) |
| search | nein | STRING | Filtert AdWords nach einem Begriff |
| regex_keyword | nein | STRING | Regex-Filter auf Keyword-Feld |
| regex_url | nein | STRING | Regex-Filter auf URL-Feld |
| regex_host | nein | STRING | Regex-Filter auf Host-Feld |
| format | nein | STRING | XML oder JSON. Default: XML |

### keyword.domain.seo
**Endpoint:** `https://api.sistrix.com/keyword.domain.seo`
**Credits:** 1 / Eintrag
**Description:** Organische Keyword-Rankings für das Objekt.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Objekt, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| date | nein | DATE | Woche, für die Keyword-Daten geholt werden |
| ai_answer | nein | BOOLEAN | true = nur Keywords mit AI Answer Box als SERP-Feature. Default: false |
| serp_feature | nein | STRING | Nur Keywords mit dem angegebenen SERP-Feature |
| limit | nein | INTEGER | Max. Anzahl organischer Keywords. Default: 100 |
| offset | nein | INTEGER | Startposition |
| search | nein | STRING | Filtert organische Keywords nach einem Begriff |
| from_pos | nein | INTEGER | Begrenzt Ergebnisse ab dieser Rankingposition |
| to_pos | nein | INTEGER | Begrenzt Ergebnisse bis zu dieser Rankingposition |
| regex_keyword | nein | STRING | Regex-Filter auf Keyword-Feld |
| regex_url | nein | STRING | Regex-Filter auf URL-Feld |
| regex_host | nein | STRING | Regex-Filter auf Host-Feld |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „Each keyword includes the keyword itself (kw), the ranking position … competition
level … overall traffic the keyword generates (traffic), and the ranking URL (url)."

### keyword.questions
**Endpoint:** `https://api.sistrix.com/keyword.questions`
**Credits:** 1 / Eintrag
**Description:** „a list of Google queries that include the specified keyword together with common question words."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land. Default: Account-Standard |
| lang | nein | STRING | Ergebnissprache (ISO 639). Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Fragen |
| page | nein | INTEGER | Seitennummer |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Eintrag `question` (vollständiger Fragetext), `amount` (Häufigkeit),
`traffic` (Suchvolumen).

### keyword.sem
**Endpoint:** `https://api.sistrix.com/keyword.sem`
**Credits:** 1 / Eintrag
**Description:** Top-AdWords-Positionen für ein Keyword.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| limit | nein | INTEGER | Max. Anzahl AdWord-Rankings |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| domain | nein | STRING | Domain, für die Top-AdWords-Positionen geholt werden |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `position` (relatives Ranking), `total_position` (allgemeine SERP-Position),
`title` und `text` des AdWord-Rankings, `displayurl`, `type`.

### keyword.seo
**Endpoint:** `https://api.sistrix.com/keyword.seo`
**Credits:** 1 / Eintrag
**Description:** Top-Positionen im organischen Ranking für das Keyword.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl organischer Rankings. Default: 100 |
| history | nein | BOOLEAN | Historische Ranking-Daten für DOMAIN oder URL. Default: false |
| domain | nein | STRING | Domain, für die Top-Positionen geholt werden |
| url | nein | STRING | URL, für die Top-Positionen geholt werden |
| date | nein | DATE | Woche für historische Ranking-Daten |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag `position`, `domain`, `url`.

### keyword.seo.competition
**Endpoint:** `https://api.sistrix.com/keyword.seo.competition`
**Credits:** 1 / Eintrag
**Description:** Competition-Wert für das/die Keyword(s).

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword(s). Bulk: `["kw1", "kw2"]` |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Liste von Keywords, je mit `kw` und `competition`.
**Notes:** POST. Unterstützt Bulk-Abfragen per Array.

### keyword.seo.metrics
**Endpoint:** `https://api.sistrix.com/keyword.seo.metrics`
**Credits:** **5 / Eintrag**
**Description:** „Displays the most important data points for the given keywords, such as competition, CPC (cost per click), traffic, clicks, and device distribution."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword(s); Bulk per Array `["kw1", "kw2"]` |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Keyword: `kw`, `competition`, `cpc`, `traffic`, `clicks`,
`mobile_distribution`, `desktop_distribution`.
**Notes:** Teuerste der keyword-Methoden (5 statt 1 pro Eintrag).

### keyword.seo.searchintent
**Endpoint:** `https://api.sistrix.com/keyword.seo.searchintent`
**Credits:** 1 / Eintrag
**Description:** Geschätzte Suchintentionen für das Keyword.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Key-Value-Paare der Intent-Typen (Know, Visit, Website, Do) „with values ranging from
0 to 100".
**Notes:** POST.

### keyword.seo.serpfeatures
**Endpoint:** `https://api.sistrix.com/keyword.seo.serpfeatures`
**Credits:** 1 / Eintrag
**Description:** „Provides information about the SERP features found for a specified keyword, including counts of various types such as featured snippets, knowledge panels, and more."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| show-all-types | nein | BOOLEAN | Alle verfügbaren SERP-Feature-Typen einbeziehen. Default: false |
| date | nein | DATE | Stichtag für Anzahl und Typen der SERPs |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Liste der SERP-Typen als Key-Value-Paare: Key = SERP-Typ, Value = Häufigkeit.

### keyword.seo.traffic
**Endpoint:** `https://api.sistrix.com/keyword.seo.traffic`
**Credits:** 1 / Eintrag
**Description:** Traffic- und Klick-Mengen für das Keyword.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Zu analysierendes Keyword |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| show-all | nein | BOOLEAN | Vollständige historische Traffic-Daten. Default: false |
| show-all-countries | nein | BOOLEAN | Traffic-/Klickdaten über alle verfügbaren Länder. Default: false |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `kw`, `traffic`, `clicks`, `country`.

### keyword.seo.traffic.estimation
**Endpoint:** `https://api.sistrix.com/keyword.seo.traffic.estimation`
**Credits:** 1 / Eintrag
**Description:** Geschätzter Traffic-Anteil je Rankingposition für ein Keyword.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: Account-Standard |
| limit | nein | INTEGER | Max. Anzahl Ranking-Schätzungen. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag `position`, `url`, `traffic_estimation` (geschätzter Traffic-Prozentsatz).
**Notes:** POST.

---

# links

4 Methoden plus Übersichtsseite. Auffällig: die Credit-Modelle unterscheiden sich hier stark
(pro Eintrag, pro Eintrag mit Deckel, Pauschale).

### links (Übersicht)
**Endpoint:** `https://api.sistrix.com/links`
**Credits:** not stated
**Description:** „Generates a list of associated link methods that can be executed."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain/host/path/url) | ja | STRING | Objekt, auf das die Methode angewendet wird |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „A JSON or XML array that lists all available link-related methods."

### links.linktargets
**Endpoint:** `https://api.sistrix.com/links.linktargets`
**Credits:** 1 / Eintrag
**Description:** Top-Linkziele für DOMAIN, HOST oder PATH.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain/host/path) | ja | STRING | „Defines the DOMAIN, PATH or HOST to which the method is applied" |
| limit | nein | INTEGER | Max. Anzahl Linkziele. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „Each link target includes the target URL (to), the number of links pointing to this
target (links), and details about the origins of these links, such as the number of different
domains (domains/dpop), the number of distinct IP addresses (ips/ipop), and the number of different
networks (nets/netpop)."

### links.linktexts
**Endpoint:** `https://api.sistrix.com/links.linktexts`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of the top backlink texts for the specified DOMAIN, PATH, or HOST, including detailed metrics about the origin of these backlink texts."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain/host/path) | ja | STRING | Domain, Host oder Path |
| limit | nein | INTEGER | Max. Anzahl Linktexte. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag: Textinhalt, Link-Counts, Host-Metriken (HPOP/HOSTS), IP-Metriken
(IPOP/IPS), Domain-Metriken (DPOP/DOMAINS), Netzwerkdaten (netpop/net).

### links.list
**Endpoint:** `https://api.sistrix.com/links.list`
**Credits:** 1 / Eintrag — **MAX 250 pro Query**
**Description:** „Returns up to 10,000 known backlinks for the given DOMAIN. The results include the source URL and target URL of each link."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain) | ja | STRING | Domain, auf die die Methode angewendet wird |
| offset | nein | INTEGER | Startpunkt in der Backlink-Liste (überspringt N Ergebnisse) |
| limit | nein | INTEGER | Max. Anzahl Backlinks. Default: 10000 |
| regex_link_from | nein | STRING | Regex-Filter auf `link_from` (URL-kodiert) |
| regex_link_to | nein | STRING | Regex-Filter auf `link_to` (URL-kodiert) |
| regex_link_text | nein | STRING | Regex-Filter auf `link_text` (URL-kodiert) |
| format | nein | STRING | XML oder JSON. Default: XML |

**Notes:** POST. Einzige dokumentierte Methode mit einem Credit-Deckel pro Query (250) trotz
Default-`limit` von 10.000.

### links.overview
**Endpoint:** `https://api.sistrix.com/links.overview`
**Credits:** **25** (Pauschale pro Call)
**Description:** „Provides an overview of the found backlinks for the given DOMAIN, HOST or PATH. The function returns the total number of backlinks, the number of unique hostnames (Host-Pop), unique domains (Domain-Pop), and unique IP addresses (IP-Pop)."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object (domain/host/path) | ja | STRING | Zu analysierende Domain, Host oder Path |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `total` (Backlinks gesamt), `hosts` (Host-Pop), `domains` (Domain-Pop), `networks`,
`c-class` (Backlinks aus verschiedenen Class-C-IP-Bereichen). Jede Kategorie nutzt einen
`num`-Parameter für die Anzahl.

---

# ai

11 Methoden. Die Wurzelfunktion `ai` erzeugt „a list of available AI methods for the given PROMPT or
ENTITY" — Parameter: `api_key` (Pflicht), `entity` (optional), `prompt` (optional), `format`
(optional). Credits: not stated.

> **Achtung `model`-Enum:** Die erlaubten Werte des `model`-Parameters rendern auf mehreren Seiten
> fehlerhaft (`aioi, mode`, `aioa, imode`, `aimodeaiochatgpt`). Sauber angegeben sind sie nur auf
> `ai.tracker.sources.urls` und `ai.entity.prompts`: **`all, chatgpt, perplexity, aio, aimode`**.
> Auf den übrigen Seiten ist das Enum **nicht belastbar dokumentiert** — nicht raten.

### ai.models
**Endpoint:** `https://api.sistrix.com/ai.models`
**Credits:** not stated (Seite zeigt „Start request (Free query)")
**Description:** „a list of all available models for ai functions"; je Eintrag Model-Code und Label des LLM.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| format | nein | STRING | XML oder JSON. Default: XML |

### ai.top.brands
**Endpoint:** `https://api.sistrix.com/ai.top.brands`
**Credits:** 1 / Eintrag
**Description:** „a ranked list of the top AI-recognized brands".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| lang | nein | STRING | Ergebnissprache nach ISO 639 (Deutsch = de). Default: Toolbox-Standardsprache |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: 1 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag Markenname, Ranking, Anzahl Datenpunkte, Sichtbarkeitsindex (`visindex`).

### ai.top.entities
**Endpoint:** `https://api.sistrix.com/ai.top.entities`
**Credits:** 1 / Eintrag
**Description:** „the most prominent entities according to AI analysis."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| lang | nein | STRING | Ergebnissprache nach ISO 639. Default: Toolbox-Standardsprache |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Entity-Name, Ranking, Anzahl Vorkommen, Sichtbarkeitsindex.
**Notes:** POST.

### ai.top.sources
**Endpoint:** `https://api.sistrix.com/ai.top.sources`
**Credits:** 1 / Eintrag
**Description:** „the most prominent sources according to AI analysis."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| lang | nein | STRING | Ergebnissprache nach ISO 639. Default: Toolbox-Standardsprache |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Website-Host, Ranking, Anzahl Vorkommen, Sichtbarkeitsindex.
**Notes:** POST.

### ai.prompt.answers
**Endpoint:** `https://api.sistrix.com/ai.prompt.answers`
**Credits:** 1 / Eintrag
**Description:** Ruft zuvor generierte AI-Antworten zu einem Prompt ab.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| prompt | ja | STRING | Prompt, zu dem Antworten geholt werden |
| model | nein | STRING | Model der Ausführung; ohne Angabe alle AI-Modelle. Default: all |
| country | nein | STRING | ISO 3166-1 alpha-2 (z. B. DE). Ohne Angabe alle Länder. Default: all |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** „Each entry provides the answer text, specifies which AI model produced it (gpt, gemini,
deepseek), and the language of the response."

### ai.entity.overview
**Endpoint:** `https://api.sistrix.com/ai.entity.overview`
**Credits:** **10 pro Call**
**Description:** „Provides an overview of the selected ENTITY, including key metrics and usage statistics."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | „a clearly identifiable object or concept mentioned within a text" |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Nutzungsmetriken und Gesamtzahl Prompts (`prompt_count`).
**Notes:** Die Seite listet 53 unterstützte Länder (DE, AT, CH, NL, FR, IT, ES, PL, UK, USA, SE, BR,
TR, BE, IE, PT, DK, NO, FI, GR, HU, SK, CZ, CA, AU, MX, RU, JP, IN, ZA, RO, SI, HR, BG, TH, VN, ID,
PE, AR, CO, CY, MT, MY, PH, NZ, AE, EG, CL, PK, SG, NG, VE, UA).

### ai.entity.competition
**Endpoint:** `https://api.sistrix.com/ai.entity.competition`
**Credits:** 1 / Eintrag
**Description:** „Creates a list of AI competitors for the given ENTITY."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | Identifizierbares Objekt/Konzept (Firmen/Marken, Personen, Orte, Produkte) |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

### ai.entity.environment
**Endpoint:** `https://api.sistrix.com/ai.entity.environment`
**Credits:** 1 / Eintrag
**Description:** „Creates a list of AI entities for the given ENTITY."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | Identifizierbares Objekt/Konzept |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Liste verwandter Entities plus Credit-Verbrauch.

### ai.entity.prompts
**Endpoint:** `https://api.sistrix.com/ai.entity.prompts`
**Credits:** 1 / Eintrag
**Description:** „Creates a list of the most relevant prompts for which the specified entity appears in the response."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | Identifizierbares Objekt/Konzept |
| model | nein | STRING | `all` / `aimode` / `aio` / `chatgpt`. Default: alle Modelle |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Eintrag Prompt, AI-Model, generierter Antworttext, Ausgabesprache.
**Notes:** Eine der zwei Seiten mit sauber gerendertem `model`-Enum.

### ai.entity.prompts.count
**Endpoint:** `https://api.sistrix.com/ai.entity.prompts.count`
**Credits:** **1 pro Call**
**Description:** „Returns the total number of AI-generated prompts where the specified entity appears in the response."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | Identifizierbares Objekt/Konzept |
| model | nein | STRING | Filtert nach AI-Model. Default: alle Modelle |
| expand_models | nein | BOOLEAN | Liste aller Modelle statt einzelner Zahl. Default: FALSE |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| format | nein | STRING | XML oder JSON. Default: XML |

**Notes:** `model`-Enum auf dieser Seite fehlerhaft gerendert — siehe Warnhinweis oben.

### ai.entity.sources
**Endpoint:** `https://api.sistrix.com/ai.entity.sources`
**Credits:** 1 / Eintrag
**Description:** „a list of hosts, domains or urls that are most similar to the specified entity".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| entity | ja | STRING | Identifizierbares Objekt/Konzept |
| model | nein | STRING | AI-Model der Ausführung. Default: alle Modelle |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| type | nein | STRING | Response-Schema: `host`, `domain` oder `url`. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Eintrag Source, Anzahl Ergebnisse, `prompt_count`.
**Notes:** `model`-Enum fehlerhaft gerendert — siehe Warnhinweis oben.

---

# ai.tracker

7 Methoden. „The **ai.tracker** functions are built around **projects**, which serve as the central
management unit for monitoring AI visibility." In Projekten werden ein fester Satz Prompts und
konkrete Wettbewerber über verschiedene AI-Modelle hinweg getrackt.

### ai.tracker.overview
**Endpoint:** `https://api.sistrix.com/ai.tracker.overview`
**Credits:** not stated (Seite zeigt „Start request (Free query)")
**Description:** „Returns a list of all PROJECTS currently stored in the account"; je Projekt Hash und Name.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| format | nein | STRING | XML oder JSON. Default: XML |

**Notes:** POST. Liefert den `project`-Hash, den alle übrigen ai.tracker-Methoden benötigen.

### ai.tracker.competitors
**Endpoint:** `https://api.sistrix.com/ai.tracker.competitors`
**Credits:** 1 / Eintrag
**Description:** „a list of competitors for a specified project, comparing project-defined competitors with additional entities mentioned by the AI."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| model | nein | STRING | AI-Model-Filter. Default: all |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Für projektdefinierte Wettbewerber `visindex`-Werte; für sonstige von der AI genannte
Entities ist `visindex` = 0. Zusätzlich Mention-Counts und Anzahl distinkter Prompts.
**Notes:** `model`-Enum fehlerhaft gerendert — siehe Warnhinweis oben.

### ai.tracker.environment
**Endpoint:** `https://api.sistrix.com/ai.tracker.environment`
**Credits:** 1 / Eintrag
**Description:** „a list of entities and terms that constitute the thematic ENVIRONMENT of the specified PROJECT."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `misc` (erkannter Begriff), `prompt_count` (Anzahl eindeutiger Prompts mit dem Begriff),
`mentions` (Gesamterwähnungen).
**Notes:** Als einzige ai.tracker-Methode **ohne** `page`-Parameter dokumentiert.

### ai.tracker.prompts
**Endpoint:** `https://api.sistrix.com/ai.tracker.prompts`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of all prompts stored in the specified PROJECT" mit Aufschlüsselung je AI-Model.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Vordefinierte Prompts des Projekts, `brand_found` (Marken-Sichtbarkeit), `avg_pos`
(Durchschnittsposition, falls die Marke genannt wird), `tags` je Prompt, `models`-Sektion mit
Verfügbarkeit über die AI-Plattformen (ChatGPT, Perplexity, AIO).

### ai.tracker.sources.domains
**Endpoint:** `https://api.sistrix.com/ai.tracker.sources.domains`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of root domains that the AI uses as sources for the prompts defined in the specified PROJECT."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| model | nein | STRING | Model der Ausführung; ohne Angabe alle AI-Modelle. Default: alle Modelle |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `domain`, `mentions`, `prompt_count`.

### ai.tracker.sources.hosts
**Endpoint:** `https://api.sistrix.com/ai.tracker.sources.hosts`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of hosts that the AI uses as sources for the prompts defined in the specified PROJECT."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| model | nein | STRING | Model der Ausführung. Default: all |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `host`, `mentions`, `prompt_count`, `brands`.
**Notes:** `model`-Enum fehlerhaft gerendert — siehe Warnhinweis oben.

### ai.tracker.sources.urls
**Endpoint:** `https://api.sistrix.com/ai.tracker.sources.urls`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of specific URLs that the AI uses as sources for the prompts defined in the specified PROJECT."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| model | nein | STRING | `all`, `chatgpt`, `perplexity`, `aio`, `aimode`. Default: all |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** URL, Erwähnungshäufigkeit, Prompt-Count, zugehörige Marken.
**Notes:** Einzige Seite mit sauber gerendertem `model`-Enum — hier als Referenz für alle
ai.*-Methoden verwendbar.

---

# ai.check

5 Methoden. „The **ai.checker** functions are designed to monitor the visibility of brands and
domains across various chatbots."

> **Bereichsweite Regel:** Bei allen ai.check-Methoden gilt „At least one of the `brands` or
> `domains` parameters must be provided." Beide akzeptieren einen Einzelwert oder ein JSON-Array
> mit **bis zu 100** Einträgen. `ai.check.sources` akzeptiert zusätzlich die Singular-Aliase
> `brand` und `domain`.

### ai.check.overview
**Endpoint:** `https://api.sistrix.com/ai.check.overview`
**Credits:** **10 pro Call**
**Description:** Liefert den analysierten Scope (Mentions, Citations oder kombiniert), Gesamtzahl
Prompts, Aufschlüsselung je AI-Model und je Land.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| brands | nein* | STRING | Ein oder mehrere Markennamen; Einzelwert oder JSON-Array (max. 100) |
| domains | nein* | STRING | Eine oder mehrere Domains; Einzelwert oder JSON-Array (max. 100) |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| format | nein | STRING | JSON oder XML. Default: XML |

\* mindestens eines von `brands` / `domains` erforderlich.

### ai.check.competitors
**Endpoint:** `https://api.sistrix.com/ai.check.competitors`
**Credits:** 1 / Eintrag
**Description:** Ergebnis enthält Markennamen und Credit-Verbrauch.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| brands | nein* | STRING | Markennamen; Einzelwert oder JSON-Array (max. 100) |
| domains | nein* | STRING | Domains; Einzelwert oder JSON-Array (max. 100) |
| model | nein | STRING | Model der Ausführung. Default: all |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: all |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

\* mindestens eines von `brands` / `domains` erforderlich.

### ai.check.prompts
**Endpoint:** `https://api.sistrix.com/ai.check.prompts`
**Credits:** 1 / Eintrag
**Description:** Ergebnis enthält Prompts, AI-Model-Info, Antworttext und Ausgabeland.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| brands | nein* | STRING | Markennamen; Einzelwert oder JSON-Array (max. 100) |
| domains | nein* | STRING | Domains; Einzelwert oder JSON-Array (max. 100) |
| model | nein | STRING | Model der Ausführung. Default: all |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

\* mindestens eines von `brands` / `domains` erforderlich.

### ai.check.prompts.count
**Endpoint:** `https://api.sistrix.com/ai.check.prompts.count`
**Credits:** 1 / Eintrag
**Description:** „The result contains one entry per history date. Each entry includes the date (date)
and the number of prompts at that date (prompt_count)."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| brands | nein* | STRING | Markennamen; Einzelwert oder JSON-Array (max. 100) |
| domains | nein* | STRING | Domains; Einzelwert oder JSON-Array (max. 100) |
| model | nein | STRING | Model der Ausführung oder `all`. Default: all |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: all |
| limit | nein | INTEGER | Anzahl zurückgegebener History-Einträge. **Default: 7** |
| format | nein | STRING | XML oder JSON. Default: XML |

\* mindestens eines von `brands` / `domains` erforderlich.

**Notes:** Ausreißer bei den Defaults: `limit` = **7** (History-Daten), nicht 100 — und **kein**
`page`-Parameter. Einzige ai.check-Methode mit diesen Abweichungen.

### ai.check.sources
**Endpoint:** `https://api.sistrix.com/ai.check.sources`
**Credits:** 1 / Eintrag
**Description:** not stated (über Parameter-/Response-Doku hinaus).

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| brands | nein* | STRING | Markennamen; Einzelwert oder JSON-Array (max. 100). Alias: `brand` |
| domains | nein* | STRING | Domains; Einzelwert oder JSON-Array (max. 100). Alias: `domain` |
| type | nein | STRING | Response-Schema: `domain`, `url` oder `host`. Default: not stated |
| model | nein | STRING | Model der Ausführung; ohne Angabe alle AI-Modelle. Default: alle Modelle |
| country | nein | STRING | ISO 3166-1 alpha-2. Default: alle Länder |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Result-Set. Default: 1 |
| format | nein | STRING | XML oder JSON. Default: XML |

\* mindestens eines von `brands` / `domains` erforderlich.

---

# project

17 Methoden. Alle akzeptieren POST gegen `https://api.sistrix.com/`. Nahezu alle benötigen den
Projekt-`hash`, der über `project.overview` beschafft wird.

> **Wichtige Einschränkung zu Credits:** Nur `project.competitors` nennt auf seiner Seite überhaupt
> Credit-Kosten. **Alle** anderen Seiten dieses Bereichs machen keine Angabe;
> `project.start.onpage.check` sagt nur, dass URL-Kosten abgezogen werden, ohne Zahl. Eine
> Kostenschätzung für eine project-API-Integration ist aus der Doku allein **nicht möglich**.

### project.overview
**Endpoint:** `https://api.sistrix.com/project.overview`
**Credits:** not stated
**Description:** Liefert alle Projekte des Accounts mit Hash und Name.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „Each project includes its identifying hash (hash) and the project name (name)."
**Notes:** Beschafft den Projekt-Hash, den fast alle übrigen `project.*`-Methoden benötigen.

### project.create
**Endpoint:** `https://api.sistrix.com/project.create`
**Credits:** not stated — aber: „Each project beyond the limit incurs additional costs" (Betrag nicht genannt)
**Description:** Legt ein neues Onpage-Projekt an.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| address_object | ja | STRING | Domain, Host, Path oder URL |
| acknowledge_extra_cost | nein | BOOLEAN | Erlaubt Überschreiten des Projektlimits; Mehrkosten. Default: false |
| crawl_limit | nein | INTEGER | Crawl-Limit; Maximum **25.000** Seiten. Default: 10.000 |
| project_name | nein | STRING | Projektname. Default: Domainname |
| frequency | nein | STRING | Crawl-Frequenz: `weekly` oder `manual`. Default: not stated |
| kw | nein | STRING | Bis zu **40** Keywords im Array-Format |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `domain`, `project_hash`, `project_name`, `crawl_limit`, `frequency`.
**Notes:** **„Only one project can be created every 20 seconds"** — schreibende Methode.

### project.competitors
**Endpoint:** `https://api.sistrix.com/project.competitors`
**Credits:** **1 / Eintrag** (einzige Methode im Bereich mit Kostenangabe)
**Description:** Wettbewerber einer Domain auf Basis der im Projekt hinterlegten Keywords.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: not stated |
| page | nein | INTEGER | Welches Wettbewerber-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Wettbewerber `domain`, `visindex`, `match` (Nähe zur analysierten Domain).
**Notes:** „The domain for which the competitors are being returned is also included in the list with
a match value of 100."

### project.visibilityindex
**Endpoint:** `https://api.sistrix.com/project.visibilityindex`
**Credits:** not stated
**Description:** Sichtbarkeitsindex für ein Projekt.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| date | nein | DATE | Stichtag für den Sichtbarkeitsindex |
| competitors | nein | BOOLEAN | TRUE = Vergleich mit den im Projekt hinterlegten Wettbewerbern. Default: false |
| tag | nein | STRING | Berechnet den Index für Keywords eines bestimmten Tags |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `domain`, `date`, `value`.

### project.ranking
**Endpoint:** `https://api.sistrix.com/project.ranking`
**Credits:** not stated
**Description:** Die im Projekt erfassten Rankings.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Max. Anzahl Rankings. Default: 100 |
| offset | nein | INTEGER | Startposition |
| tag | nein | STRING | Filter nach einem oder mehreren Tags (Trennzeichen `\|`) |
| date | nein | DATE | Historische Rankings (Format YYYY-MM-DD) |
| regex_keyword | nein | STRING | Regex-Filter auf Keywords (URL-kodiert) |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Gesamtzahl Rankings, Keyword, Position, URL, Tags, Device, Land, generierter Traffic,
verwendete Suchmaschine.

### project.keyword.serps
**Endpoint:** `https://api.sistrix.com/project.keyword.serps`
**Credits:** not stated
**Description:** SERP-Daten für ein Keyword innerhalb eines Projekts.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| kw | ja | STRING | Keyword, auf das die Methode angewendet wird |
| country | nein | STRING | Land der Ausführung. Default: Standardland des Toolbox-Accounts |
| city | nein | STRING | Filtert die SERPs nach Stadt |
| device | nein | STRING | Filtert die SERPs nach Gerät |
| searchengine | nein | STRING | Filtert die SERPs nach Suchmaschine. Default: Google |
| date | nein | DATE | Stichtag für historische Ergebnisse (ISO 8601: YYYY-MM-DD) |
| format | nein | STRING | XML oder JSON. Default: XML |

**Notes:** „If no SERP data is available for the specified date, the function will return an error."

### project.external.links
**Endpoint:** `https://api.sistrix.com/project.external.links`
**Credits:** not stated
**Description:** „Returns a list of all links to external sites your project links to."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: not stated |
| page | nein | INTEGER | Welches Link-Set. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** „Each link includes the content type and the status code of the link destination."

### project.onpage
**Endpoint:** `https://api.sistrix.com/project.onpage`
**Credits:** not stated
**Description:** „Provides detailed information about a project, including the project name and associated tags".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash (via `project.overview`) |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Projekt-Hash, Projektname, Liste der Tags mit Namen und Hashes.

### project.onpage.overview
**Endpoint:** `https://api.sistrix.com/project.onpage.overview`
**Credits:** not stated
**Description:** „Returns a overview of on-page crawl data for the project… a list of completed crawls, with each entry providing the timestamp, the total number of pages crawled, and a summary of errors, warnings, or notes detected during that specific crawl."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Crawl `time`, `pages`, `error`, `warnings`, `notice`.
**Notes:** Liefert die verfügbaren Crawl-Daten für den `date`-Parameter von `project.onpage.crawl`.

### project.onpage.crawl
**Endpoint:** `https://api.sistrix.com/project.onpage.crawl`
**Credits:** not stated
**Description:** Liste der bei einem Crawl gefundenen Fehler und Warnungen.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| date | nein | DATE | Datum des Onpage-Crawls (ISO 8601, YYYY-MM-DD). Default: aktueller Crawl |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Issue `type` (error oder warning), `name`, `count`.
**Notes:** „This function offers an overview of the issues detected, but does not include detailed
information on individual errors." Die hier zurückgegebenen Namen sind die gültigen Werte für den
`issue`-Parameter von `project.onpage.issue`.

### project.onpage.issue
**Endpoint:** `https://api.sistrix.com/project.onpage.issue`
**Credits:** not stated
**Description:** Detailinformationen zu einem einzelnen Issue eines Onpage-Crawls.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| issue | ja | STRING | Das Issue, zu dem Details geholt werden |
| date | nein | DATE | Datum des Onpage-Crawls (ISO 8601) |
| limit | nein | INTEGER | Max. Anzahl Issues. Default: 100 |
| offset | nein | INTEGER | Startposition |
| format | nein | STRING | XML oder JSON. Default: XML |

**Notes:** „Possible values for the ISSUE parameter can be obtained using the project.onpage.crawl function."

### project.onpage.urls
**Endpoint:** `https://api.sistrix.com/project.onpage.urls`
**Credits:** not stated
**Description:** „Provides a comprehensive list of all HTML pages crawled within the project".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Max. Anzahl gecrawlter Seiten. Default: 100 |
| offset | nein | INTEGER | Startposition |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `hash`, `name`, `count`; je URL URL-Score, Title, Größe (Bytes) „and more".
**Notes:** Nutzt `offset` statt `page` — abweichend von den links/resources/cookies-Methoden.

### project.onpage.links
**Endpoint:** `https://api.sistrix.com/project.onpage.links`
**Credits:** not stated
**Description:** Links aus dem Onpage-Crawl des Projekts.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Link-Set |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `hash`, `name`, `count`; je Link Quelle (`from`) und Ziel (`to`), Typ, Statuscode,
Linktext, Content-Type sowie Follow-Status: **1 = follow, 0 = nofollow, -1 = unbestimmt**.

### project.onpage.resources
**Endpoint:** `https://api.sistrix.com/project.onpage.resources`
**Credits:** not stated
**Description:** Vom Crawl gefundene Ressourcen.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Ressourcen-Set |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `hash`, `name`, `count`; je Ressource `type`, `url`, `status_code`, `size` (Bytes).

### project.onpage.resources.usage
**Endpoint:** `https://api.sistrix.com/project.onpage.resources.usage`
**Credits:** not stated
**Description:** „detailed information on how and where each discovered resource (such as images, JavaScript, or CSS files) is utilized".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Ressourcen-Set |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Ressourcen-Typ, URL, Statuscode, Größe, Antwortzeit. Bei Bild-Ressourcen zusätzlich
Alt-Text und Title-Attribut.
**Notes:** POST.

### project.onpage.cookies
**Endpoint:** `https://api.sistrix.com/project.onpage.cookies`
**Credits:** not stated
**Description:** Beim Onpage-Crawl erkannte Cookies.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| limit | nein | INTEGER | Ergebnisse pro Seite. Default: 100 |
| page | nein | INTEGER | Welches Set |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Projekt-Hash, Projektname, Gesamtzahl Cookies; je Cookie Name, Wert, Ablaufdatum,
Pfad, HTTP-only-Tag, Secure-Flag, SameSite-Attribute, Anzahl der Cookie nutzenden URLs.
**Notes:** „The SISTRIX Onpage Crawler captures all cookies that the web server intends to set, but
does not actually use them."

### project.start.onpage.check
**Endpoint:** `https://api.sistrix.com/project.start.onpage.check`
**Credits:** keine Zahl genannt — „The costs for the URLs used in this crawl will be deducted from your SISTRIX account"
**Description:** Startet einen Onpage-Crawl für ein Projekt.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| project | ja | STRING | Eindeutiger Projekt-Hash |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `name` und `hash` des Projekts, für das der Crawl gestartet wurde. Bei Erfolg keine
Fehlermeldung.
**Notes:** **Schreibend und kostenwirksam.** Ein Test-Aufruf wird von der Doku nicht unterstützt —
Aufrufe haben reale Kostenfolgen.

---

# marketplace

17 Methoden, Amazon-Marketplace-Daten. Abweichend vom Rest der API ist der `country`-Default hier
durchgängig **`de`** (nicht das Account-Standardland). Die Seiten listen 50+ Marketplace-Länder
(DE, AT, CH, NL, FR, IT, ES, PL, UK, USA, SE, BR, TR, BE, IE, PT, DK, NO, FI, GR, HU, SK, CZ, CA,
AU, MX, RU, JP, IN, ZA, RO, SI, HR, BG, TH, VN, ID, PE, AR, CO, CY, MT, MY, PH, NZ, AE, EG, CL, PK,
SG, NG, VE, UA).

**Credit-Modelle im Bereich, drei Stufen:** die meisten Methoden 1/Eintrag; `marketplace.product.price`
und `marketplace.product.reviews` **1 + 1/Eintrag**; `marketplace.product.overview` und
`marketplace.visibility.list` pauschal **1 pro Call**. `marketplace.product` nennt keine Kosten.

### marketplace.product
**Endpoint:** `https://api.sistrix.com/marketplace.product`
**Credits:** not stated
**Description:** Erzeugt für eine ASIN „a list of associated marketplace methods that can be executed".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | „An identifier used by Amazon to uniquely identify a product." |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Methode `method`, `url`, `name`.

### marketplace.product.overview
**Endpoint:** `https://api.sistrix.com/marketplace.product.overview`
**Credits:** **1 pro Call**
**Description:** „Provides an overview of the most important data for an Amazon product".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | Amazon-Produkt-ID |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** `asin`, `country`, `title`, `listed_since`, `price`, `currency`, `price_link`,
`reviews`, `avg_review`, `reviews_link`, Keyword-Count, `keyword_link`.
**Notes:** Liefert Verweise/Links auf verwandte API-Funktionen für die Detailanalyse.

### marketplace.product.keywords
**Endpoint:** `https://api.sistrix.com/marketplace.product.keywords`
**Credits:** 1 / Eintrag
**Description:** „provides the keywords, the product's ranking (position in the search results) for each keyword, and the traffic amount."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | Amazon-Produkt-ID |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| limit | nein | INTEGER | Max. Anzahl Keywords |
| offset | nein | INTEGER | Startpunkt in der Keyword-Liste |
| type | nein | STRING | `organic`, `sponsored` oder `global`. Default: organic |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag Keyword, Ranking, Traffic, Competition-Level (Skala 0–100).

### marketplace.product.bestsellers
**Endpoint:** `https://api.sistrix.com/marketplace.product.bestsellers`
**Credits:** 1 / Eintrag
**Description:** Bestseller-Rang-Informationen für ein oder mehrere Produkte.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin / asins | ja | STRING | Produkt-ID(s); für mehrere `asins` im Format `["ASIN1", "ASIN2", ...]` |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Eintrag ASIN, Rang in der Bestseller-Kategorie, Kategorie-ID, Kategoriename.

### marketplace.product.price
**Endpoint:** `https://api.sistrix.com/marketplace.product.price`
**Credits:** **1 + 1/Eintrag**
**Description:** Preisentwicklung eines Produkts.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | Amazon-Produkt-ID |
| country | nein | STRING/countrycode | Land der Preisanzeige. Default: `de` |
| start | nein | DATE | Startdatum, ab dem die Preisentwicklung geholt wird |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** `lowest`, `lowest_date`, `highest`, `highest_date` sowie historische Preisdaten mit
`date_from`, `date_to`, Preiswert, Seller-ID und Seller-Name.
**Notes:** POST.

### marketplace.product.reviews
**Endpoint:** `https://api.sistrix.com/marketplace.product.reviews`
**Credits:** **1 + 1/Eintrag**
**Description:** Review-Daten eines Produkts.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | Amazon-Produkt-ID |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| limit | nein | INTEGER | Max. Anzahl historischer Reviews |
| offset | nein | INTEGER | Startpunkt in der Review-Liste |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Gesamtzahl Reviews, Durchschnittsbewertung (`avg_score`); je Review `score`,
Reviewer-Username, Datum, Titel — „in descending order by date".
**Notes:** POST.

### marketplace.category.products
**Endpoint:** `https://api.sistrix.com/marketplace.category.products`
**Credits:** 1 / Eintrag
**Description:** „top products based on bestseller ranking" einer Kategorie.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| category | ja | STRING | Amazon-Kategorie-ID aus der SISTRIX-URL |
| brand | nein | STRING | Filter nach Markenname |
| bb_seller_id | nein | STRING | Filter nach einzelner Seller-ID |
| min_bb_price | nein | NUMERIC | Minimaler Buy-Box-Preis (Dezimaltrenner Punkt) |
| max_bb_price | nein | NUMERIC | Maximaler Buy-Box-Preis |
| min_bsr | nein | INTEGER | Minimaler Bestseller-Rang |
| max_bsr | nein | INTEGER | Maximaler Bestseller-Rang |
| limit | nein | INTEGER | Max. Anzahl Produkte (Basis 100, bis 10.000). Default: 100 |
| offset | nein | INTEGER | Startpunkt für Pagination |
| country | nein | STRING/countrycode | Marketplace-Ländercode. Default: `de` |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Produkt `rank`, `asin`, `parent_asin`, `title`, `rev_count`, `rev_stars`, `price`,
`sales`, `sales_rank`.

### marketplace.brand.sellers
**Endpoint:** `https://api.sistrix.com/marketplace.brand.sellers`
**Credits:** 1 / Eintrag
**Description:** „Returns a list of sellers associated with a specific brand, including basic details such as the number of products and visibility indices."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| id / name | ja | STRING | Marken-Identifier (ID oder Name) von Amazon |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| limit | nein | INTEGER | Max. Anzahl Seller. Default: not stated |
| offset | nein | INTEGER | Startpunkt in der Seller-Liste. Default: not stated |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Je Seller Name, Seller-ID, Sichtbarkeitsindizes (organic, sponsored, global),
Produktanzahl der Marke, durchschnittliches Review-Rating, Buybox-Prozentsatz.

### marketplace.keyword.environment
**Endpoint:** `https://api.sistrix.com/marketplace.keyword.environment`
**Credits:** 1 / Eintrag
**Description:** „products, sellers, brands, or categories found in the environment of a specified keyword". Das Keyword-Umfeld umfasst Begriffe, die das gesuchte Keyword enthalten.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Suchbegriff für die Umfeldanalyse |
| country | nein | STRING/countrycode | Marketplace-Ländercode. Default: `de` |
| limit | nein | INTEGER | Max. Anzahl Ergebnisse. Default: 100 |
| offset | nein | INTEGER | Startpunkt in der Ergebnisliste |
| type | nein | STRING | Umfeldtyp: `organic`, `sponsored` oder `global`. Default: organic |
| kw_order | nein | STRING | Sortierung: `traffic` oder `all` (Keyword-Ähnlichkeit). Default: traffic |
| kw_limit | nein | INTEGER | Begrenzt die fürs Umfeld berücksichtigten Keywords |
| top_3 | nein | BOOLEAN | Nur Top-3-Ranking-Keywords |
| object | nein | STRING | Rückgabetyp: `product`, `seller`, `category` oder `brand`. Default: product |
| high_traffic | nein | BOOLEAN | Nur Keywords mit Traffic ≥ 100 |
| format | nein | STRING | JSON oder XML. Default: XML |

**Response:** Identifier, Titel, Anzahl Keyword-Vorkommen, Rating, durchschnittliches Mindest-Ranking,
Traffic-Score in Prozent.

### marketplace.keyword.search.ideas
**Endpoint:** `https://api.sistrix.com/marketplace.keyword.search.ideas`
**Credits:** 1 / Eintrag
**Description:** Keyword-Ideen zu einem Suchbegriff.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Suchbegriff |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| limit | nein | INTEGER | Anzahl Keyword-Ideen (max. 10.000). Default: 100 |
| offset | nein | INTEGER | Startpunkt in der Keyword-Liste |
| mode | nein | STRING | Matching: `include` (enthält Wort), `same` (alle Wörter, beliebige Reihenfolge), `exact` (gleiche Reihenfolge). Default: `include` |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Keyword `keyword`, `traffic`, `competition`, `date`.

### marketplace.keyword.traffic
**Endpoint:** `https://api.sistrix.com/marketplace.keyword.traffic`
**Credits:** 1 / Eintrag
**Description:** Traffic für einen Suchbegriff.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Suchbegriff, für den der Traffic geliefert wird |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| date | nein | DATE | Einzelner historischer Datenpunkt (ISO 8601, YYYY-MM-DD) |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Je Keyword `kw` und `traffic`.

### marketplace.sales.overview
**Endpoint:** `https://api.sistrix.com/marketplace.sales.overview`
**Credits:** 1 / Eintrag
**Description:** Überblick über Verkaufsmetriken für Produkte, Seller oder Marken.

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin / id | ja | STRING | Amazon-Identifier für Produkt, Seller oder Marke |
| country | nein | STRING/countrycode | Marketplace-Ländercode. Default: `de` |
| object | nein | STRING | Suchtyp: `product`, `seller` oder `brand`. Default: `product` |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Objekt-Identifier (`asin` bei Produkten, `id` bei Sellern), `estimated_sales`,
`estimated_sales_rank`, `estimated_sales_volume`, `estimated_sales_volume_rank`.
**Notes:** Brand-ID und Seller-ID lassen sich aus SISTRIX-URLs entnehmen.

### marketplace.serp.history
**Endpoint:** `https://api.sistrix.com/marketplace.serp.history`
**Credits:** 1 / Eintrag
**Description:** „a list of SERPs for the given keyword".

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| kw | ja | STRING | Suchbegriff für die SERP-Historie |
| from | nein | DATE | Frühester Monat im Format YYYY-MM |
| to | nein | DATE | Spätester Monat im Format YYYY-MM |
| num_dates | nein | INTEGER | Anzahl verschiedener Daten für die SERP-Anzeige. Default: 1 |
| details | nein | BOOLEAN | Produkttitel und Markenname mitliefern. Default: false |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Fundatum, Ranking-Position, Kennzeichnung organic/sponsored, Produkt-ASIN. Mit
`details=true` zusätzlich Produkttitel und Markeninformation.
**Notes:** Einzige Methode mit `from`/`to` im **YYYY-MM**-Format (nicht ISO-8601-Tagesdatum).

### marketplace.visibility.list
**Endpoint:** `https://api.sistrix.com/marketplace.visibility.list`
**Credits:** **1 pro Call**
**Description:** „returns the total visibility index for the products specified in the list. You can filter the results by historical data, as well as by organic and sponsored visibility."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| hash | ja | STRING | Eindeutiger Hash einer SISTRIX-Produktliste |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| type | nein | STRING | Index-Typ: `organic`, `sponsored` oder `global`. Default: organic |
| date | nein | DATE | Historischer Datenpunkt (ISO 8601, YYYY-MM-DD) |
| vi_type | nein | STRING | Berechnung auf Basis `seller` oder `brand`. Default: not stated |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Zwei Elemente: Datum der Index-Berechnung und der numerische Sichtbarkeitsindex-Wert.

### marketplace.visindex.product
**Endpoint:** `https://api.sistrix.com/marketplace.visindex.product`
**Credits:** 1 / Eintrag
**Description:** „Provides the SISTRIX visibility index for Amazon for a specific product."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| asin | ja | STRING | Amazon-Produkt-ID |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| history | nein | BOOLEAN | TRUE = alle verfügbaren historischen Werte. Default: FALSE |
| type | nein | STRING | Index-Typ (organic/sponsored/global). Default: organic |
| date | nein | DATE | Einzelner historischer Datenpunkt (ISO 8601, YYYY-MM-DD) |
| limit | nein | INTEGER | Begrenzt historische Werte. Default: 30 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `product-asin`, `product-title`, `date`, `value`.

### marketplace.visindex.brand
**Endpoint:** `https://api.sistrix.com/marketplace.visindex.brand`
**Credits:** 1 / Eintrag
**Description:** „Provides the SISTRIX visibility index for Amazon for a specific brand."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| id | ja | STRING | Amazon-Marken-ID aus der SISTRIX-URL |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| history | nein | BOOLEAN | TRUE = alle verfügbaren historischen Werte. Default: FALSE |
| type | nein | STRING | `organic`, `sponsored` oder `global`. Default: `organic` |
| date | nein | DATE | Einzelner historischer Datenpunkt (ISO 8601, YYYY-MM-DD) |
| limit | nein | INTEGER | Begrenzt historische Werte. Default: 30 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** `brand-id`, `brand-name`, `date`, `value`.

### marketplace.visindex.seller
**Endpoint:** `https://api.sistrix.com/marketplace.visindex.seller`
**Credits:** 1 / Eintrag
**Description:** „Indicates the visibility of the seller's products on Amazon and helps assess their performance in search results."

| Parameter | Pflicht | Typ | Beschreibung |
|---|---|---|---|
| api_key | ja | STRING | Persönlicher API-Key |
| id | ja | STRING | „An identifier used by Amazon to uniquely distinguish a seller." |
| country | nein | STRING/countrycode | Land der Ausführung. Default: `de` |
| history | nein | BOOLEAN | TRUE = alle verfügbaren historischen Werte. Default: FALSE |
| type | nein | STRING | `organic`, `sponsored` oder `global`. Default: **not stated** |
| date | nein | DATE | Einzelner historischer Datenpunkt (ISO 8601, YYYY-MM-DD) |
| limit | nein | INTEGER | Begrenzt historische Werte. Default: 30 |
| format | nein | STRING | XML oder JSON. Default: XML |

**Response:** Seller-ID, Seller-Name, Datum, Sichtbarkeitsindex-Wert je Eintrag.
**Notes:** Der `type`-Default ist auf dieser Seite **nicht** angegeben — die Schwestermethoden
`visindex.brand`/`visindex.product` nennen `organic`, das wurde hier bewusst **nicht** übernommen.

---

## Crawl-Log

Alle bisher abgerufenen URLs. **Bislang keine einzige Seite unerreichbar.**

### Einstieg / Querschnitt
| URL | Status |
|---|---|
| https://www.sistrix.com/api/ | OK |
| https://www.sistrix.com/api/overview/ | OK |
| https://www.sistrix.com/api/basic-functions/ | OK |
| https://www.sistrix.com/api/limitations/ | OK |
| https://www.sistrix.com/api/errors/ | OK |
| https://www.sistrix.com/api/basic-functions/countries/ | OK |
| https://www.sistrix.com/api/basic-functions/credits/ | OK |
| https://www.sistrix.com/api/basic-functions/serpfeatures/ | OK |

### domain (17/17 OK)
| URL | Status |
|---|---|
| https://www.sistrix.com/api/domain/ | OK |
| https://www.sistrix.com/api/domain/domain-competitors-sem/ | OK |
| https://www.sistrix.com/api/domain/domain-competitors-seo/ | OK |
| https://www.sistrix.com/api/domain/domain-hosts/ | OK |
| https://www.sistrix.com/api/domain/domain-ideas/ | OK |
| https://www.sistrix.com/api/domain/domain-kwcount-sem/ | OK |
| https://www.sistrix.com/api/domain/domain-kwcount-seo/ | OK |
| https://www.sistrix.com/api/domain/domain-kwcount-seo-top10/ | OK |
| https://www.sistrix.com/api/domain/domain-opportunities/ | OK |
| https://www.sistrix.com/api/domain/domain-overview/ | OK |
| https://www.sistrix.com/api/domain/domain-paths/ | OK |
| https://www.sistrix.com/api/domain/domain-ranking-distribution/ | OK |
| https://www.sistrix.com/api/domain/domain-traffic-estimation/ | OK |
| https://www.sistrix.com/api/domain/domain-urlcount-seo/ | OK |
| https://www.sistrix.com/api/domain/domain-urls/ | OK |
| https://www.sistrix.com/api/domain/domain-visibilityindex/ | OK |
| https://www.sistrix.com/api/domain/domain-visibilityindex-overview/ | OK |

### keyword + links (17/17 OK)
| URL | Status |
|---|---|
| https://www.sistrix.com/api/keyword/ | OK |
| https://www.sistrix.com/api/keyword/keyword-domain-sem/ | OK |
| https://www.sistrix.com/api/keyword/keyword-domain-seo/ | OK |
| https://www.sistrix.com/api/keyword/keyword-questions/ | OK |
| https://www.sistrix.com/api/keyword/keyword-sem/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-competition/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-metrics/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-searchintent/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-serpfeatures/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-traffic/ | OK |
| https://www.sistrix.com/api/keyword/keyword-seo-traffic-estimation/ | OK |
| https://www.sistrix.com/api/links/ | OK |
| https://www.sistrix.com/api/links/links-linktargets/ | OK |
| https://www.sistrix.com/api/links/links-linktexts/ | OK |
| https://www.sistrix.com/api/links/links-list/ | OK |
| https://www.sistrix.com/api/links/links-overview/ | OK |

### ai / ai.tracker / ai.check (26/26 OK)
| URL | Status |
|---|---|
| https://www.sistrix.com/api/ai/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-competition/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-environment/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-overview/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-prompts/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-prompts-count/ | OK |
| https://www.sistrix.com/api/ai/ai-entity-sources/ | OK |
| https://www.sistrix.com/api/ai/ai-models/ | OK |
| https://www.sistrix.com/api/ai/ai-prompt-answers/ | OK |
| https://www.sistrix.com/api/ai/ai-top-brands/ | OK |
| https://www.sistrix.com/api/ai/ai-top-entities/ | OK |
| https://www.sistrix.com/api/ai/ai-top-sources/ | OK |
| https://www.sistrix.com/api/ai-tracker/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-competitors/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-environment/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-overview/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-prompts/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-sources-domains/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-sources-hosts/ | OK |
| https://www.sistrix.com/api/ai-tracker/ai-tracker-sources-urls/ | OK |
| https://www.sistrix.com/api/ai-check/ | OK |
| https://www.sistrix.com/api/ai-check/ai-check-competitors/ | OK |
| https://www.sistrix.com/api/ai-check/ai-check-overview/ | OK |
| https://www.sistrix.com/api/ai-check/ai-check-prompts/ | OK |
| https://www.sistrix.com/api/ai-check/ai-check-prompts-count/ | OK |
| https://www.sistrix.com/api/ai-check/ai-check-sources/ | OK |

### project (18/18 OK)
| URL | Status |
|---|---|
| https://www.sistrix.com/api/project/ | OK |
| https://www.sistrix.com/api/project/project-competitors/ | OK |
| https://www.sistrix.com/api/project/project-create/ | OK |
| https://www.sistrix.com/api/project/project-external-links/ | OK |
| https://www.sistrix.com/api/project/project-keyword-serps/ | OK |
| https://www.sistrix.com/api/project/project-onpage/ | OK |
| https://www.sistrix.com/api/project/project-onpage-cookies/ | OK |
| https://www.sistrix.com/api/project/project-onpage-crawl/ | OK |
| https://www.sistrix.com/api/project/project-onpage-issue/ | OK |
| https://www.sistrix.com/api/project/project-onpage-links/ | OK |
| https://www.sistrix.com/api/project/project-onpage-overview/ | OK |
| https://www.sistrix.com/api/project/project-onpage-resources/ | OK |
| https://www.sistrix.com/api/project/project-onpage-resources-usage/ | OK |
| https://www.sistrix.com/api/project/project-onpage-urls/ | OK |
| https://www.sistrix.com/api/project/project-overview/ | OK |
| https://www.sistrix.com/api/project/project-ranking/ | OK |
| https://www.sistrix.com/api/project/project-start-onpage-check/ | OK |
| https://www.sistrix.com/api/project/project-visibilityindex/ | OK |

### marketplace (18/18 OK)
| URL | Status |
|---|---|
| https://www.sistrix.com/api/marketplace/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-brand-sellers/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-category-products/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-keyword-environment/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-keyword-search-ideas/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-keyword-traffic/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product-bestsellers/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product-keywords/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product-overview/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product-price/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-product-reviews/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-sales-overview/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-serp-history/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-visibility-list/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-visindex-brand/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-visindex-product/ | OK |
| https://www.sistrix.com/api/marketplace/marketplace-visindex-seller/ | OK |

### Gesamt
**104 URLs gecrawlt — 104× OK, 0 unerreichbar.**
(8 Einstieg/Querschnitt + 17 domain + 17 keyword/links + 26 ai/ai.tracker/ai.check + 18 project
+ 18 marketplace.)

### Nicht gecrawlt (bewusst, kein API-Inhalt)
Die folgenden von `/api/` verlinkten Seiten behandeln Client-Anbindungen, keine API-Methoden:
`/api/connection-to-google-sheets/`, `/api/connection-to-microsoft-excel/`,
`/api/connection-to-chatbot-ai/` (MCP-Server), `/api/connection-to-google-data-studio/`.

## Offene Punkte / Unklar

- **Keine literalen Beispiel-Responses:** SISTRIX rendert Beispieldaten nur mit gültigem API-Key
  („Show example data"). Response-Strukturen sind daher aus den Feldbeschreibungen dokumentiert —
  das gilt für **alle 91 Methoden**, ohne Ausnahme. Für echte Payload-Beispiele wäre eine
  authentifizierte Browser-Session nötig.
- **Fehler-Response-Format:** Codes und Meldungen sind dokumentiert, die XML-/JSON-Hülle nicht.
- **Länderliste:** nicht statisch dokumentiert, nur über `countries` zur Laufzeit abrufbar.
  (Die ai- und marketplace-Seiten listen jeweils ~53 Länder im Fließtext — siehe dort.)
- **SERP-Feature-Liste:** nicht statisch dokumentiert, nur über `serpfeatures` abrufbar.
- **Credits der Basisfunktionen:** nicht ausgewiesen; laut Getting-Started-Regel damit kostenlos.
- **Credits im project-Bereich:** bis auf `project.competitors` (1/Eintrag) **durchgängig nicht
  angegeben**. Eine Kostenschätzung für eine project-Integration ist aus der Doku nicht möglich —
  hier ist eine Rückfrage bei SISTRIX oder eine Messung über den `credits`-Zähler im Response nötig.
- **`model`-Enum im ai-Bereich:** rendert auf mehreren Seiten fehlerhaft
  (`aioi, mode` / `aioa, imode` / `aimodeaiochatgpt`). Belastbar nur auf
  `ai.tracker.sources.urls` und `ai.entity.prompts`: `all, chatgpt, perplexity, aio, aimode`.
  Für die übrigen Seiten wurde das Enum bewusst **nicht** übernommen.
- **Zwei unklare Defaults im marketplace-Bereich:** `vi_type` bei `marketplace.visibility.list` und
  `type` bei `marketplace.visindex.seller` — die jeweiligen Seiten nennen keinen Default. Bei
  `visindex.seller` liegt `organic` nahe (Schwestermethoden), wurde aber nicht übernommen.
- **Schreibende/kostenwirksame Methoden** (für ein MCP-Design relevant): `project.create`
  (max. 1 Projekt / 20 Sekunden, Mehrkosten über Limit) und `project.start.onpage.check`
  (zieht URL-Kosten, kein Test-Modus).
