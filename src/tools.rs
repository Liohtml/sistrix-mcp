//! Curated tool catalog and argument dispatch.
//!
//! Instead of auto-generating one tool per SISTRIX API method (90+ tools that
//! flood the model's context), we expose ~17 hand-crafted tools that cover the
//! common SEO questions, plus `sistrix_api` as an escape hatch for the full API.
//!
//! SISTRIX methods take their subject as one of the query parameters `domain`,
//! `host`, `path`, or `url` (the docs call this the "address object"). Tools
//! model that as a `target` string plus a `scope` enum that picks the
//! parameter name.

use std::collections::HashMap;
use std::sync::Arc;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{json, Map, Value};

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    String,
    Integer,
    Boolean,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Requirement {
    Required,
    Optional,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: ParamKind,
    pub requirement: Requirement,
    pub default: Option<&'static str>,
    pub choices: &'static [&'static str],
    /// String params only: also accept an array of strings, serialized in
    /// SISTRIX's bulk format `["a", "b"]`.
    pub accepts_list: bool,
}

impl ParamSpec {
    const fn new(name: &'static str, kind: ParamKind, description: &'static str) -> Self {
        Self {
            name,
            description,
            kind,
            requirement: Requirement::Optional,
            default: None,
            choices: &[],
            accepts_list: false,
        }
    }

    const fn list_ok(mut self) -> Self {
        self.accepts_list = true;
        self
    }

    const fn required(mut self) -> Self {
        self.requirement = Requirement::Required;
        self
    }

    const fn default_value(mut self, value: &'static str) -> Self {
        self.default = Some(value);
        self
    }

    const fn choices(mut self, values: &'static [&'static str]) -> Self {
        self.choices = values;
        self
    }
}

/// One selectable report inside a tool (value of the tool's select argument).
#[derive(Debug, Clone)]
pub struct SelectCase {
    pub value: &'static str,
    pub method: &'static str,
    pub fixed: &'static [(&'static str, &'static str)],
    /// Optional-in-schema arguments this case cannot work without.
    pub requires: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub enum Binding {
    /// One fixed SISTRIX method.
    Fixed {
        method: &'static str,
        fixed: &'static [(&'static str, &'static str)],
    },
    /// Method chosen by the value of `arg` (first case is the default).
    Select {
        arg: &'static str,
        cases: &'static [SelectCase],
    },
    /// Generic escape hatch: expects `method` and `params` arguments.
    Raw,
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    /// Human-readable display name shown by MCP clients and registries.
    pub title: &'static str,
    pub description: &'static str,
    /// When set, the tool takes `target` + `scope`; the slice lists the
    /// allowed scope values (SISTRIX parameter names), first is the default.
    pub target_scopes: Option<&'static [&'static str]>,
    pub params: Vec<ParamSpec>,
    pub binding: Binding,
}

/// A fully resolved SISTRIX API call.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    pub method: String,
    pub params: Vec<(String, String)>,
}

// ---------------------------------------------------------------------------
// Common parameters
// ---------------------------------------------------------------------------

const SCOPE_FULL: &[&str] = &["domain", "host", "path", "url"];
const SCOPE_DHP: &[&str] = &["domain", "host", "path"];

const fn p_country() -> ParamSpec {
    ParamSpec::new(
        "country",
        ParamKind::String,
        "Country code for the SISTRIX country index, e.g. 'de', 'at', 'fr', 'us'. \
         List all codes with sistrix_lists. Defaults to the account's default country.",
    )
}

const fn p_limit(default: &'static str) -> ParamSpec {
    ParamSpec::new(
        "limit",
        ParamKind::Integer,
        "Maximum number of rows to return.",
    )
    .default_value(default)
}

const fn p_offset() -> ParamSpec {
    ParamSpec::new(
        "offset",
        ParamKind::Integer,
        "Number of rows to skip (pagination start position).",
    )
}

const fn p_page() -> ParamSpec {
    ParamSpec::new(
        "page",
        ParamKind::Integer,
        "Result page to return (pagination).",
    )
}

const fn p_date() -> ParamSpec {
    ParamSpec::new(
        "date",
        ParamKind::String,
        "Date in YYYY-MM-DD format to fetch historic values for. Defaults to the latest data.",
    )
}

const fn p_kw() -> ParamSpec {
    ParamSpec::new("kw", ParamKind::String, "The keyword to analyze.")
}

const fn p_ai_model() -> ParamSpec {
    ParamSpec::new(
        "model",
        ParamKind::String,
        "Restrict results to one AI model.",
    )
    .choices(&["all", "chatgpt", "perplexity", "aio", "aimode"])
}

fn select_param(
    name: &'static str,
    description: &'static str,
    cases: &'static [SelectCase],
) -> ParamSpec {
    let mut p = ParamSpec::new(name, ParamKind::String, description);
    p.default = Some(cases[0].value);
    p
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

macro_rules! cases {
    ($($value:literal => $method:literal $([$(($fk:literal, $fv:literal)),+])? $(requires $req:expr)?),+ $(,)?) => {
        &[$(SelectCase {
            value: $value,
            method: $method,
            fixed: &[$($(($fk, $fv)),+)?],
            requires: cases!(@req $($req)?),
        }),+]
    };
    (@req) => { &[] };
    (@req $req:expr) => { $req };
}

pub fn catalog() -> Vec<ToolSpec> {
    const LIST_CASES: &[SelectCase] = cases! {
        "countries" => "countries",
        "serp_features" => "serpfeatures",
        "ai_models" => "ai.models",
    };
    const VISIBILITY_CASES: &[SelectCase] = cases! {
        "current" => "domain.visibilityindex",
        "history" => "domain.visibilityindex" [("history", "true")],
        "daily" => "domain.visibilityindex" [("daily", "true")],
        "min_max" => "domain.visibilityindex.overview",
    };
    const KWCOUNT_CASES: &[SelectCase] = cases! {
        "seo" => "domain.kwcount.seo",
        "seo_top10" => "domain.kwcount.seo.top10",
        "ads" => "domain.kwcount.sem",
        "top_urls" => "domain.urlcount.seo",
    };
    const COMPETITOR_CASES: &[SelectCase] = cases! {
        "seo" => "domain.competitors.seo",
        "ads" => "domain.competitors.sem",
    };
    const IDEA_CASES: &[SelectCase] = cases! {
        "opportunities" => "domain.opportunities",
        "related_searches" => "domain.ideas",
    };
    const STRUCTURE_CASES: &[SelectCase] = cases! {
        "top_urls" => "domain.urls",
        "top_hosts" => "domain.hosts",
        "top_paths" => "domain.paths",
        "ranking_distribution" => "domain.ranking.distribution",
        "traffic_by_path" => "domain.traffic.estimation",
    };
    const DOMAIN_RANKING_CASES: &[SelectCase] = cases! {
        "seo" => "keyword.domain.seo",
        "ads" => "keyword.domain.sem",
    };
    const KEYWORD_CASES: &[SelectCase] = cases! {
        "rankings" => "keyword.seo",
        "metrics" => "keyword.seo.metrics",
        "competition" => "keyword.seo.competition",
        "search_intent" => "keyword.seo.searchintent",
        "serp_features" => "keyword.seo.serpfeatures",
        "traffic" => "keyword.seo.traffic",
        "traffic_estimation" => "keyword.seo.traffic.estimation",
        "questions" => "keyword.questions",
        "ads" => "keyword.sem",
    };
    const LINKS_CASES: &[SelectCase] = cases! {
        "overview" => "links.overview",
        "backlinks" => "links.list",
        "link_texts" => "links.linktexts",
        "link_targets" => "links.linktargets",
    };
    const AI_TOP_CASES: &[SelectCase] = cases! {
        "brands" => "ai.top.brands",
        "entities" => "ai.top.entities",
        "sources" => "ai.top.sources",
    };
    const AI_ENTITY_CASES: &[SelectCase] = cases! {
        "overview" => "ai.entity.overview",
        "competition" => "ai.entity.competition",
        "environment" => "ai.entity.environment",
        "prompts" => "ai.entity.prompts",
        "prompt_count" => "ai.entity.prompts.count",
        "sources" => "ai.entity.sources",
    };
    const AI_TRACKER_CASES: &[SelectCase] = cases! {
        "projects" => "ai.tracker.overview",
        "competitors" => "ai.tracker.competitors" requires &["project"],
        "environment" => "ai.tracker.environment" requires &["project"],
        "prompts" => "ai.tracker.prompts" requires &["project"],
        "source_domains" => "ai.tracker.sources.domains" requires &["project"],
        "source_hosts" => "ai.tracker.sources.hosts" requires &["project"],
        "source_urls" => "ai.tracker.sources.urls" requires &["project"],
    };
    const PROJECT_CASES: &[SelectCase] = cases! {
        "list" => "project.overview",
        "visibility" => "project.visibilityindex" requires &["project"],
        "rankings" => "project.ranking" requires &["project"],
        "competitors" => "project.competitors" requires &["project"],
        "onpage_overview" => "project.onpage.overview" requires &["project"],
        "keyword_serps" => "project.keyword.serps" requires &["project", "kw"],
    };
    const AMAZON_CASES: &[SelectCase] = cases! {
        "product_overview" => "marketplace.product.overview",
        "product_keywords" => "marketplace.product.keywords",
        "product_price" => "marketplace.product.price",
        "product_reviews" => "marketplace.product.reviews",
    };

    vec![
        ToolSpec {
            name: "sistrix_credits",
            title: "API credit balance",
            description: "Show the remaining SISTRIX API credits for this account. Free to call. \
                          Credits refill weekly; most other tools cost credits per returned row.",
            target_scopes: None,
            params: vec![],
            binding: Binding::Fixed {
                method: "credits",
                fixed: &[],
            },
        },
        ToolSpec {
            name: "sistrix_lists",
            title: "Discovery lists",
            description: "Discovery lists, free to call: available country codes for the Google \
                          and Amazon indices, available SERP-feature names (for filters), and \
                          available AI model codes.",
            target_scopes: None,
            params: vec![select_param("list", "Which list to fetch.", LIST_CASES)],
            binding: Binding::Select {
                arg: "list",
                cases: LIST_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_domain_overview",
            title: "Domain SEO overview",
            description: "One-call overview of a domain's most important SEO key figures: \
                          visibility index, organic keyword count, and ads count, each with date. \
                          Costs a flat 5 credits. The go-to first look at any domain.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![p_country()],
            binding: Binding::Fixed {
                method: "domain.overview",
                fixed: &[],
            },
        },
        ToolSpec {
            name: "sistrix_visibility",
            title: "Visibility Index",
            description: "SISTRIX Visibility Index for a domain/host/path/URL: the current value, \
                          weekly history, daily values for the last 30 days, or the all-time \
                          high/low (report=min_max, flat 10 credits; others 1 credit per value).",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param(
                    "report",
                    "Which visibility report to fetch.",
                    VISIBILITY_CASES,
                ),
                p_country(),
                p_date(),
                p_limit("52"),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: VISIBILITY_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_keyword_counts",
            title: "Ranking keyword counts",
            description: "How many keywords a domain/host/path/URL ranks for: organic keywords, \
                          top-10 organic keywords, Google Ads keywords, or the count of ranking \
                          URLs. Set history=true for weekly time series.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param("report", "Which count to fetch.", KWCOUNT_CASES),
                p_country(),
                p_date(),
                ParamSpec::new(
                    "history",
                    ParamKind::Boolean,
                    "Return historic weekly counts instead of just the latest value.",
                ),
                p_limit("52"),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: KWCOUNT_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_competitors",
            title: "SEO & Ads competitors",
            description: "Competitors of a domain and their similarity in % — organic search \
                          competitors (channel=seo) or Google Ads competitors (channel=ads). \
                          1 credit per row.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param("channel", "Competitor channel.", COMPETITOR_CASES),
                p_country(),
                p_limit("20"),
            ],
            binding: Binding::Select {
                arg: "channel",
                cases: COMPETITOR_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_keyword_ideas",
            title: "Keyword opportunities & ideas",
            description: "Keyword research for a domain: ranking opportunities (keywords just \
                          off page 1 with a 0-100 'gain' potential) or related-search keyword \
                          ideas from Google's 'related queries'. 1 credit per row.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param("report", "Which idea source to use.", IDEA_CASES),
                p_country(),
                p_limit("25"),
                ParamSpec::new(
                    "regex_keyword",
                    ParamKind::String,
                    "Only ideas matching this regular expression (server-side filter). \
                     report=related_searches only.",
                ),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: IDEA_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_domain_structure",
            title: "Ranking structure",
            description: "Where a domain's rankings come from: top ranking URLs, hosts \
                          (subdomains), or paths (directories) with top-10/top-100 counts and \
                          visibility share; the keyword distribution across Google result pages; \
                          or the top paths by estimated traffic. 1 credit per row.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param(
                    "report",
                    "Which structural report to fetch.",
                    STRUCTURE_CASES,
                ),
                p_country(),
                p_date(),
                p_limit("25"),
                p_offset(),
                ParamSpec::new(
                    "regex_url",
                    ParamKind::String,
                    "Only URLs matching this regular expression (server-side filter). \
                     report=top_urls only.",
                ),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: STRUCTURE_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_domain_rankings",
            title: "Domain keyword rankings",
            description: "All keywords a domain/host/path/URL ranks for, with position, traffic \
                          and ranking URL — organic (channel=seo) or Google Ads (channel=ads). \
                          Filter by position range, search term, or SERP feature. 1 credit per \
                          row — keep the limit tight.",
            target_scopes: Some(SCOPE_FULL),
            params: vec![
                select_param("channel", "Ranking channel.", DOMAIN_RANKING_CASES),
                p_country(),
                p_date(),
                p_limit("25"),
                p_offset(),
                ParamSpec::new(
                    "search",
                    ParamKind::String,
                    "Only keywords containing this term.",
                ),
                ParamSpec::new(
                    "regex_keyword",
                    ParamKind::String,
                    "Only keywords matching this regular expression. Filtering happens \
                     server-side, so only matching rows cost credits.",
                ),
                ParamSpec::new(
                    "regex_url",
                    ParamKind::String,
                    "Only rankings whose URL matches this regular expression \
                     (server-side filter).",
                ),
                ParamSpec::new(
                    "from_pos",
                    ParamKind::Integer,
                    "Only rankings at or below this position (e.g. 11 for page 2 and worse). \
                     channel=seo only.",
                ),
                ParamSpec::new(
                    "to_pos",
                    ParamKind::Integer,
                    "Only rankings at or above this position (e.g. 20). channel=seo only.",
                ),
                ParamSpec::new(
                    "serp_feature",
                    ParamKind::String,
                    "Only keywords showing this SERP feature (names via sistrix_lists). \
                     channel=seo only.",
                ),
                ParamSpec::new(
                    "ai_answer",
                    ParamKind::Boolean,
                    "Only keywords with an AI answer box in the SERP. channel=seo only.",
                ),
            ],
            binding: Binding::Select {
                arg: "channel",
                cases: DOMAIN_RANKING_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_keyword",
            title: "Keyword analysis",
            description: "Everything about one keyword: top organic rankings, key metrics \
                          (volume/CPC/competition/device split — 5 credits per keyword!), \
                          competition score, search intent (Know/Visit/Website/Do), SERP \
                          features, traffic, estimated click share per position, common \
                          questions, or Google Ads results.",
            target_scopes: None,
            params: vec![
                select_param("report", "Which keyword report to fetch.", KEYWORD_CASES),
                ParamSpec::new(
                    "kw",
                    ParamKind::String,
                    "The keyword to analyze. For report='metrics' and \
                     report='competition' an array of keywords is accepted for a bulk \
                     lookup in a single call.",
                )
                .required()
                .list_ok(),
                p_country(),
                p_limit("25"),
                ParamSpec::new(
                    "domain",
                    ParamKind::String,
                    "Restrict report=rankings/ads to this domain's positions.",
                ),
                p_date(),
                ParamSpec::new(
                    "lang",
                    ParamKind::String,
                    "Result language for report=questions (ISO 639, e.g. 'de').",
                ),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: KEYWORD_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_links",
            title: "Backlink profile",
            description: "Backlink data for a domain/host/path: profile overview (total links, \
                          host/domain/IP/network popularity — flat 25 credits), the backlink \
                          list (1 credit per row, max 250 per query), top link texts, or top \
                          link targets.",
            target_scopes: Some(SCOPE_DHP),
            params: vec![
                select_param("report", "Which backlink report to fetch.", LINKS_CASES),
                p_limit("25"),
                p_offset(),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: LINKS_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_ai_top",
            title: "AI visibility top charts",
            description: "SISTRIX AI-visibility charts: the brands, entities, or source domains \
                          most often referenced in AI answers (ChatGPT, Perplexity, Google AI \
                          Overviews, AI Mode). 1 credit per row.",
            target_scopes: None,
            params: vec![
                select_param("report", "Which AI top list to fetch.", AI_TOP_CASES),
                ParamSpec::new(
                    "lang",
                    ParamKind::String,
                    "Result language (ISO 639, e.g. 'de'). Defaults to the account language.",
                ),
                p_limit("25"),
                p_page(),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: AI_TOP_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_ai_entity",
            title: "AI entity analysis",
            description:
                "How AI models see one entity (brand, product, person): overview (flat 10 \
                          credits), competing entities, thematic environment, the prompts that \
                          mention it (or just their count), and the sources AI cites for it \
                          (each 1 credit per row).",
            target_scopes: None,
            params: vec![
                select_param("report", "Which entity report to fetch.", AI_ENTITY_CASES),
                ParamSpec::new(
                    "entity",
                    ParamKind::String,
                    "The entity to analyze, e.g. a brand name.",
                )
                .required(),
                p_ai_model(),
                ParamSpec::new("lang", ParamKind::String, "Result language (ISO 639)."),
                p_limit("25"),
                p_page(),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: AI_ENTITY_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_ai_tracker",
            title: "AI visibility tracker",
            description: "AI-visibility tracking projects: list the account's tracker projects \
                          (report=projects, free), then per project the tracked prompts with \
                          brand visibility, competitors, thematic environment, and the domains/\
                          hosts/URLs AI models cite as sources. Get the project hash via \
                          report=projects first.",
            target_scopes: None,
            params: vec![
                select_param("report", "Which tracker report to fetch.", AI_TRACKER_CASES),
                ParamSpec::new(
                    "project",
                    ParamKind::String,
                    "Project hash (required for every report except 'projects'; find it via \
                     report=projects).",
                ),
                p_ai_model(),
                p_limit("25"),
                p_page(),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: AI_TRACKER_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_project",
            title: "Optimizer projects",
            description: "SISTRIX Optimizer projects: list projects (report=list, free), then \
                          per project the visibility index, tracked keyword rankings, \
                          competitors, onpage-crawl overview, or the SERPs for one tracked \
                          keyword. Get the project hash via report=list first.",
            target_scopes: None,
            params: vec![
                select_param("report", "Which project report to fetch.", PROJECT_CASES),
                ParamSpec::new(
                    "project",
                    ParamKind::String,
                    "Project hash (required for every report except 'list'; find it via \
                     report=list).",
                ),
                p_kw(),
                p_date(),
                p_limit("25"),
                p_offset(),
                ParamSpec::new(
                    "tag",
                    ParamKind::String,
                    "Filter tracked keywords by tag (report=rankings/visibility).",
                ),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: PROJECT_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_amazon",
            title: "Amazon marketplace data",
            description: "Amazon marketplace data for a product (by ASIN): key-figures overview \
                          (flat 1 credit), ranking keywords with position and traffic, price \
                          history, or review history. Country defaults to amazon.de.",
            target_scopes: None,
            params: vec![
                select_param("report", "Which Amazon report to fetch.", AMAZON_CASES),
                ParamSpec::new("asin", ParamKind::String, "The Amazon product's ASIN.").required(),
                ParamSpec::new(
                    "country",
                    ParamKind::String,
                    "Amazon marketplace country code (e.g. 'de', 'us', 'uk'). Default: 'de'.",
                ),
                p_limit("25"),
            ],
            binding: Binding::Select {
                arg: "report",
                cases: AMAZON_CASES,
            },
        },
        ToolSpec {
            name: "sistrix_api",
            title: "Raw SISTRIX API call",
            description: "Escape hatch: call ANY SISTRIX API method directly. Prefer the \
                          dedicated sistrix_* tools; use this for methods they don't cover \
                          (domain.ideas filters, ai.check.*, ai.prompt.answers, \
                          marketplace.keyword.*, project.onpage.*, ...). Methods and parameters: \
                          https://www.sistrix.com/api/ — watch the credit costs. Caution: \
                          project.create and project.start.onpage.check WRITE to the account \
                          and incur costs — only call them when the user explicitly asks.",
            target_scopes: None,
            params: vec![
                ParamSpec::new(
                    "method",
                    ParamKind::String,
                    "API method in dot notation, e.g. 'domain.overview', 'links.list', or \
                     'credits'.",
                )
                .required(),
                ParamSpec::new(
                    "params",
                    ParamKind::Object,
                    "Query parameters using SISTRIX's native names, e.g. \
                     {\"domain\": \"example.com\", \"country\": \"de\", \"limit\": 10}.",
                ),
            ],
            binding: Binding::Raw,
        },
    ]
}

// ---------------------------------------------------------------------------
// Registry: schema building + dispatch
// ---------------------------------------------------------------------------

pub struct Registry {
    specs: Vec<ToolSpec>,
    index: HashMap<&'static str, usize>,
    mcp_tools: Vec<Tool>,
    default_country: Option<String>,
}

impl Registry {
    pub fn new(default_country: Option<String>) -> Self {
        let specs = catalog();
        let index = specs.iter().enumerate().map(|(i, s)| (s.name, i)).collect();
        let mcp_tools = specs
            .iter()
            .map(|s| build_mcp_tool(s, default_country.as_deref()))
            .collect();
        Self {
            specs,
            index,
            mcp_tools,
            default_country,
        }
    }

    pub fn mcp_tools(&self) -> Vec<Tool> {
        self.mcp_tools.clone()
    }

    pub fn tool_count(&self) -> usize {
        self.specs.len()
    }

    /// Resolve a tool call into a concrete SISTRIX API invocation.
    pub fn resolve(
        &self,
        tool_name: &str,
        args: &Map<String, Value>,
    ) -> Result<Invocation, String> {
        let spec = self
            .index
            .get(tool_name)
            .map(|&i| &self.specs[i])
            .ok_or_else(|| format!("unknown tool '{tool_name}'"))?;

        match &spec.binding {
            Binding::Raw => resolve_raw(args),
            Binding::Fixed { method, fixed } => {
                let mut params = self.collect_params(spec, args, None)?;
                params.extend(self.resolve_target(spec, args)?);
                Ok(Invocation {
                    method: (*method).to_string(),
                    params: apply_fixed(params, fixed),
                })
            }
            Binding::Select { arg, cases } => {
                let requested = match args.get(*arg) {
                    Some(Value::String(s)) => s.as_str(),
                    Some(other) => {
                        return Err(format!("argument '{arg}' must be a string, got {other}"))
                    }
                    None => cases[0].value,
                };
                let case = cases.iter().find(|c| c.value == requested).ok_or_else(|| {
                    format!(
                        "invalid value '{requested}' for '{arg}'. Valid values: {}",
                        cases.iter().map(|c| c.value).collect::<Vec<_>>().join(", ")
                    )
                })?;

                // Bulk keyword arrays are only documented for the SISTRIX
                // methods that support them.
                if spec.name == "sistrix_keyword"
                    && matches!(args.get("kw"), Some(Value::Array(_)))
                    && !matches!(
                        case.method,
                        "keyword.seo.metrics" | "keyword.seo.competition"
                    )
                {
                    return Err(format!(
                        "a keyword array is only supported for {arg}='metrics' or \
                         {arg}='competition'; {arg}='{requested}' needs a single keyword \
                         string"
                    ));
                }

                for required in case.requires {
                    let missing = match args.get(*required) {
                        None | Some(Value::Null) => true,
                        Some(Value::String(s)) => s.trim().is_empty(),
                        _ => false,
                    };
                    if missing {
                        return Err(format!(
                            "argument '{required}' is required when {arg}='{requested}'. \
                             {}",
                            requires_hint(spec.name, required)
                        ));
                    }
                }

                let mut params = self.collect_params(spec, args, Some(*arg))?;
                params.extend(self.resolve_target(spec, args)?);
                Ok(Invocation {
                    method: case.method.to_string(),
                    params: apply_fixed(params, case.fixed),
                })
            }
        }
    }

    /// Turn `target` + `scope` into the SISTRIX address parameter.
    fn resolve_target(
        &self,
        spec: &ToolSpec,
        args: &Map<String, Value>,
    ) -> Result<Vec<(String, String)>, String> {
        let Some(scopes) = spec.target_scopes else {
            return Ok(vec![]);
        };

        let target = match args.get("target") {
            Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
            _ => {
                return Err(
                    "required argument 'target' is missing (the domain, host, path, or URL \
                     to analyze, e.g. 'example.com')"
                        .to_string(),
                )
            }
        };

        let scope = match args.get("scope") {
            Some(Value::String(s)) => s.as_str(),
            None | Some(Value::Null) => scopes[0],
            Some(other) => return Err(format!("argument 'scope' must be a string, got {other}")),
        };
        if !scopes.contains(&scope) {
            return Err(format!(
                "invalid value '{scope}' for 'scope'. Valid values: {}",
                scopes.join(", ")
            ));
        }

        Ok(vec![(scope.to_string(), normalize_target(&target, scope)?)])
    }

    fn collect_params(
        &self,
        spec: &ToolSpec,
        args: &Map<String, Value>,
        skip: Option<&str>,
    ) -> Result<Vec<(String, String)>, String> {
        let mut out = Vec::new();

        for param in &spec.params {
            if skip == Some(param.name) {
                continue;
            }

            let value = match args.get(param.name) {
                Some(Value::Null) | None => None,
                Some(v) => {
                    let text = stringify(param, v)?;
                    // Every documented SISTRIX flag defaults to FALSE, and the
                    // API may treat a flag as set by its mere presence — so an
                    // explicit `false` is only safe when omitted entirely.
                    if param.kind == ParamKind::Boolean && text == "false" {
                        None
                    } else {
                        Some(text)
                    }
                }
            };

            match (value, param.requirement) {
                (Some(v), _) => out.push((param.name.to_string(), v)),
                (None, Requirement::Required) => {
                    return Err(format!("required argument '{}' is missing", param.name))
                }
                (None, Requirement::Optional) => {
                    if param.name == "country" {
                        if let Some(country) = &self.default_country {
                            out.push(("country".to_string(), country.clone()));
                        }
                    } else if let Some(default) = param.default {
                        out.push((param.name.to_string(), default.to_string()));
                    }
                }
            }
        }

        Ok(out)
    }
}

/// Models frequently pass a full URL even for the domain/host scopes, but
/// SISTRIX expects a bare hostname there — strip scheme, path, and query.
/// The path/url scopes are passed through untouched.
fn normalize_target(target: &str, scope: &str) -> Result<String, String> {
    if scope != "domain" && scope != "host" {
        return Ok(target.to_string());
    }
    let stripped = target
        .strip_prefix("https://")
        .or_else(|| target.strip_prefix("http://"))
        .unwrap_or(target);
    let host = stripped
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('.');
    if host.is_empty() {
        return Err(format!(
            "invalid target '{target}' for scope '{scope}': expected a hostname \
             like 'example.com'"
        ));
    }
    Ok(host.to_string())
}

fn requires_hint(tool: &str, required: &str) -> String {
    match (tool, required) {
        ("sistrix_ai_tracker", "project") => {
            "List project hashes first with report='projects'.".to_string()
        }
        ("sistrix_project", "project") => {
            "List project hashes first with report='list'.".to_string()
        }
        (_, "kw") => "Pass the tracked keyword to fetch SERPs for.".to_string(),
        _ => String::new(),
    }
}

/// Fixed parameters always win over user-supplied ones.
fn apply_fixed(
    mut params: Vec<(String, String)>,
    fixed: &[(&'static str, &'static str)],
) -> Vec<(String, String)> {
    for (k, v) in fixed {
        params.retain(|(name, _)| name != k);
        params.push(((*k).to_string(), (*v).to_string()));
    }
    params
}

/// Parameters the model must never override on the raw tool.
const RESERVED: &[&str] = &["api_key", "format"];

fn resolve_raw(args: &Map<String, Value>) -> Result<Invocation, String> {
    let method = args
        .get("method")
        .and_then(Value::as_str)
        .ok_or("required argument 'method' is missing")?
        .trim();

    let valid = !method.is_empty()
        && method.split('.').all(|seg| {
            !seg.is_empty() && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        });
    if !valid {
        return Err(format!(
            "invalid method '{method}': expected dot notation like 'domain.overview' or 'credits'"
        ));
    }

    let mut params = Vec::new();
    if let Some(raw) = args.get("params") {
        let obj = raw
            .as_object()
            .ok_or("argument 'params' must be a JSON object")?;
        for (key, value) in obj {
            if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(key)) {
                continue;
            }
            let text = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => continue,
                other => other.to_string(),
            };
            params.push((key.clone(), text));
        }
    }

    Ok(Invocation {
        method: method.to_string(),
        params,
    })
}

fn stringify(param: &ParamSpec, value: &Value) -> Result<String, String> {
    let fail = |expected: &str| {
        Err(format!(
            "argument '{}' must be {expected}, got: {value}",
            param.name
        ))
    };

    let text = match (param.kind, value) {
        (ParamKind::Integer, Value::Number(n)) if n.is_i64() || n.is_u64() => n.to_string(),
        // Some clients serialize integers as whole floats (e.g. 25.0).
        (ParamKind::Integer, Value::Number(n)) => match n.as_f64() {
            Some(f) if f.fract() == 0.0 && f.abs() < 9e15 => format!("{}", f as i64),
            _ => return fail("an integer"),
        },
        (ParamKind::Integer, Value::String(s)) if s.trim().parse::<i64>().is_ok() => {
            s.trim().to_string()
        }
        (ParamKind::Integer, _) => return fail("an integer"),

        (ParamKind::Boolean, Value::Bool(b)) => b.to_string(),
        (ParamKind::Boolean, Value::String(s)) => match s.to_ascii_lowercase().as_str() {
            "true" | "1" => "true".to_string(),
            "false" | "0" => "false".to_string(),
            _ => return fail("a boolean"),
        },
        (ParamKind::Boolean, _) => return fail("a boolean"),

        (ParamKind::String, Value::String(s)) => s.clone(),
        (ParamKind::String, Value::Number(n)) => n.to_string(),
        // SISTRIX bulk format: a JSON array of strings, e.g. ["kw1", "kw2"].
        (ParamKind::String, Value::Array(items)) if param.accepts_list => {
            let strings: Option<Vec<&str>> = items.iter().map(Value::as_str).collect();
            match strings {
                Some(list) if !list.is_empty() => {
                    serde_json::to_string(&list).expect("string list serializes")
                }
                _ => return fail("a string or a non-empty array of strings"),
            }
        }
        (ParamKind::String, _) if param.accepts_list => {
            return fail("a string or an array of strings")
        }
        (ParamKind::String, _) => return fail("a string"),

        (ParamKind::Object, v) => v.to_string(),
    };

    if !param.choices.is_empty() && !param.choices.contains(&text.as_str()) {
        return Err(format!(
            "invalid value '{text}' for '{}'. Valid values: {}",
            param.name,
            param.choices.join(", ")
        ));
    }

    Ok(text)
}

fn build_mcp_tool(spec: &ToolSpec, default_country: Option<&str>) -> Tool {
    let mut properties = Map::new();
    let mut required = Vec::new();

    if let Some(scopes) = spec.target_scopes {
        let mut target = Map::new();
        target.insert("type".into(), json!("string"));
        target.insert(
            "description".into(),
            json!(
                "The object to analyze: a domain ('example.com'), host \
                 ('www.example.com'), path ('https://example.com/blog/'), or full URL — \
                 must match the chosen scope."
            ),
        );
        properties.insert("target".to_string(), Value::Object(target));
        required.push(json!("target"));

        let mut scope = Map::new();
        scope.insert("type".into(), json!("string"));
        scope.insert("enum".into(), json!(scopes));
        scope.insert("default".into(), json!(scopes[0]));
        scope.insert(
            "description".into(),
            json!(
                "How SISTRIX should interpret the target. 'domain' aggregates all hosts \
                 (example.com incl. www); 'host' is one subdomain; 'path' a directory; \
                 'url' a single page."
            ),
        );
        properties.insert("scope".to_string(), Value::Object(scope));
    }

    for param in &spec.params {
        let mut prop = Map::new();

        if param.accepts_list {
            prop.insert("type".into(), json!(["string", "array"]));
            prop.insert("items".into(), json!({"type": "string"}));
        } else {
            let json_type = match param.kind {
                ParamKind::String => "string",
                ParamKind::Integer => "integer",
                ParamKind::Boolean => "boolean",
                ParamKind::Object => "object",
            };
            prop.insert("type".into(), json!(json_type));
        }

        let description = match (param.name, default_country) {
            ("country", Some(c)) => {
                format!("{} Configured default: '{c}'.", param.description)
            }
            _ => param.description.to_string(),
        };
        prop.insert("description".into(), json!(description));

        // The select argument's choices come from the binding cases.
        let select_choices: Option<Vec<&str>> = match &spec.binding {
            Binding::Select { arg, cases } if *arg == param.name => {
                Some(cases.iter().map(|c| c.value).collect())
            }
            _ => None,
        };
        if let Some(choices) = select_choices {
            prop.insert("enum".into(), json!(choices));
        } else if !param.choices.is_empty() {
            prop.insert("enum".into(), json!(param.choices));
        }

        if let Some(default) = param.default {
            let default_json = match param.kind {
                ParamKind::Integer => default
                    .parse::<i64>()
                    .map(Value::from)
                    .unwrap_or_else(|_| json!(default)),
                _ => json!(default),
            };
            prop.insert("default".into(), default_json);
        }

        if param.kind == ParamKind::Object {
            prop.insert("additionalProperties".into(), json!(true));
        }

        properties.insert(param.name.to_string(), Value::Object(prop));

        if param.requirement == Requirement::Required {
            required.push(json!(param.name));
        }
    }

    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".into(), Value::Array(required));
    }

    // Every curated tool is a read-only query against the SISTRIX API. The
    // raw escape hatch can reach the few write methods (project.create,
    // project.start.onpage.check), which add data but never destroy any.
    let read_only = !matches!(spec.binding, Binding::Raw);
    let annotations = ToolAnnotations {
        title: Some(spec.title.to_string()),
        read_only_hint: Some(read_only),
        destructive_hint: (!read_only).then_some(false),
        idempotent_hint: None,
        open_world_hint: Some(true),
    };

    Tool {
        name: spec.name.into(),
        description: Some(spec.description.into()),
        input_schema: Arc::new(schema),
        annotations: Some(annotations),
        icons: None,
        meta: None,
        output_schema: None,
        title: Some(spec.title.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(json: Value) -> Map<String, Value> {
        json.as_object().unwrap().clone()
    }

    #[test]
    fn catalog_names_are_unique_and_short() {
        let specs = catalog();
        let mut names: Vec<_> = specs.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), specs.len(), "duplicate tool names");
        for name in names {
            assert!(
                name.len() < 64,
                "tool name '{name}' too long for MCP clients"
            );
            assert!(name.starts_with("sistrix_"), "tool '{name}' missing prefix");
        }
    }

    #[test]
    fn fixed_tool_resolves_target_and_defaults() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "example.com"})),
            )
            .unwrap();
        assert_eq!(inv.method, "domain.overview");
        assert!(inv
            .params
            .contains(&("domain".into(), "example.com".into())));
    }

    #[test]
    fn scope_picks_the_address_parameter() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_visibility",
                &args(json!({"target": "www.example.com", "scope": "host"})),
            )
            .unwrap();
        assert_eq!(inv.method, "domain.visibilityindex");
        assert!(inv
            .params
            .contains(&("host".into(), "www.example.com".into())));
        assert!(!inv.params.iter().any(|(k, _)| k == "domain"));
    }

    #[test]
    fn invalid_scope_is_rejected_with_options() {
        let registry = Registry::new(None);
        let err = registry
            .resolve(
                "sistrix_links",
                &args(json!({"target": "example.com", "scope": "url"})),
            )
            .unwrap_err();
        assert!(err.contains("url"));
        assert!(err.contains("domain, host, path"));
    }

    #[test]
    fn missing_target_is_helpful() {
        let registry = Registry::new(None);
        let err = registry
            .resolve("sistrix_visibility", &args(json!({})))
            .unwrap_err();
        assert!(err.contains("target"));
        assert!(err.contains("example.com"));
    }

    #[test]
    fn select_tool_dispatches_and_pins_fixed_params() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_visibility",
                &args(json!({"target": "example.com", "report": "history"})),
            )
            .unwrap();
        assert_eq!(inv.method, "domain.visibilityindex");
        assert!(inv.params.contains(&("history".into(), "true".into())));
    }

    #[test]
    fn select_tool_defaults_to_first_case() {
        let registry = Registry::new(None);
        let inv = registry.resolve("sistrix_lists", &args(json!({}))).unwrap();
        assert_eq!(inv.method, "countries");
    }

    #[test]
    fn select_tool_rejects_unknown_case_with_options() {
        let registry = Registry::new(None);
        let err = registry
            .resolve(
                "sistrix_keyword",
                &args(json!({"kw": "test", "report": "galaxy"})),
            )
            .unwrap_err();
        assert!(err.contains("galaxy"));
        assert!(err.contains("rankings"));
    }

    #[test]
    fn default_country_is_injected() {
        let registry = Registry::new(Some("de".into()));
        let inv = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "example.com"})),
            )
            .unwrap();
        assert!(inv.params.contains(&("country".into(), "de".into())));
    }

    #[test]
    fn explicit_country_wins_over_default() {
        let registry = Registry::new(Some("de".into()));
        let inv = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "example.com", "country": "fr"})),
            )
            .unwrap();
        assert!(inv.params.contains(&("country".into(), "fr".into())));
        assert!(!inv.params.contains(&("country".into(), "de".into())));
    }

    #[test]
    fn no_country_param_without_default() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "example.com"})),
            )
            .unwrap();
        assert!(!inv.params.iter().any(|(k, _)| k == "country"));
    }

    #[test]
    fn tools_without_country_param_ignore_default() {
        let registry = Registry::new(Some("de".into()));
        let inv = registry
            .resolve("sistrix_credits", &args(json!({})))
            .unwrap();
        assert!(inv.params.is_empty());
    }

    #[test]
    fn case_requirements_are_enforced() {
        let registry = Registry::new(None);

        let err = registry
            .resolve("sistrix_ai_tracker", &args(json!({"report": "prompts"})))
            .unwrap_err();
        assert!(err.contains("project"));
        assert!(err.contains("projects"));

        let err = registry
            .resolve(
                "sistrix_project",
                &args(json!({"report": "keyword_serps", "project": "abc123"})),
            )
            .unwrap_err();
        assert!(err.contains("kw"));

        // The list cases need no project.
        assert!(registry
            .resolve("sistrix_ai_tracker", &args(json!({})))
            .is_ok());
        assert!(registry
            .resolve("sistrix_project", &args(json!({})))
            .is_ok());
    }

    #[test]
    fn required_keyword_is_enforced() {
        let registry = Registry::new(None);
        let err = registry
            .resolve("sistrix_keyword", &args(json!({})))
            .unwrap_err();
        assert!(err.contains("kw"));
    }

    #[test]
    fn url_targets_are_normalized_for_domain_and_host_scopes() {
        let registry = Registry::new(None);

        let inv = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "https://example.com/blog/?q=1"})),
            )
            .unwrap();
        assert!(inv
            .params
            .contains(&("domain".into(), "example.com".into())));

        let inv = registry
            .resolve(
                "sistrix_visibility",
                &args(json!({"target": "http://www.example.com/", "scope": "host"})),
            )
            .unwrap();
        assert!(inv
            .params
            .contains(&("host".into(), "www.example.com".into())));

        // path/url scopes keep the target untouched.
        let inv = registry
            .resolve(
                "sistrix_visibility",
                &args(json!({"target": "https://example.com/blog/", "scope": "path"})),
            )
            .unwrap();
        assert!(inv
            .params
            .contains(&("path".into(), "https://example.com/blog/".into())));

        let err = registry
            .resolve(
                "sistrix_domain_overview",
                &args(json!({"target": "https://"})),
            )
            .unwrap_err();
        assert!(err.contains("invalid target"));
    }

    #[test]
    fn false_flags_are_omitted() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_keyword_counts",
                &args(json!({"target": "example.com", "history": false})),
            )
            .unwrap();
        assert!(
            !inv.params.iter().any(|(k, _)| k == "history"),
            "explicit false flag must be omitted, got: {:?}",
            inv.params
        );

        let inv = registry
            .resolve(
                "sistrix_keyword_counts",
                &args(json!({"target": "example.com", "history": "False"})),
            )
            .unwrap();
        assert!(!inv.params.iter().any(|(k, _)| k == "history"));
    }

    #[test]
    fn whole_float_limits_are_accepted() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_competitors",
                &args(json!({"target": "example.com", "limit": 10.0})),
            )
            .unwrap();
        assert!(inv.params.contains(&("limit".into(), "10".into())));

        let err = registry
            .resolve(
                "sistrix_competitors",
                &args(json!({"target": "example.com", "limit": 10.5})),
            )
            .unwrap_err();
        assert!(err.contains("integer"));
    }

    #[test]
    fn boolean_and_numeric_string_coercion() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_keyword_counts",
                &args(json!({
                    "target": "example.com",
                    "history": "true",
                    "limit": "10"
                })),
            )
            .unwrap();
        assert!(inv.params.contains(&("history".into(), "true".into())));
        assert!(inv.params.contains(&("limit".into(), "10".into())));
    }

    #[test]
    fn raw_tool_passes_params_and_strips_reserved() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_api",
                &args(json!({
                    "method": "domain.ideas",
                    "params": {
                        "domain": "example.com",
                        "api_key": "evil",
                        "FORMAT": "xml",
                        "limit": 5,
                        "history": true
                    }
                })),
            )
            .unwrap();
        assert_eq!(inv.method, "domain.ideas");
        assert!(inv
            .params
            .contains(&("domain".into(), "example.com".into())));
        assert!(inv.params.contains(&("limit".into(), "5".into())));
        assert!(inv.params.contains(&("history".into(), "true".into())));
        assert!(!inv
            .params
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("api_key")));
        assert!(!inv
            .params
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("format")));
    }

    #[test]
    fn raw_tool_accepts_single_segment_methods() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve("sistrix_api", &args(json!({"method": "credits"})))
            .unwrap();
        assert_eq!(inv.method, "credits");
    }

    #[test]
    fn raw_tool_validates_method_shape() {
        let registry = Registry::new(None);
        for bad in ["", ".", "domain..overview", "bad chars", "a.b!", ".credits"] {
            let err = registry
                .resolve("sistrix_api", &args(json!({"method": bad})))
                .unwrap_err();
            assert!(
                err.contains("method"),
                "expected method error for '{bad}', got: {err}"
            );
        }
    }

    #[test]
    fn bulk_keywords_work_for_metrics_and_competition_only() {
        let registry = Registry::new(None);

        let inv = registry
            .resolve(
                "sistrix_keyword",
                &args(json!({"report": "metrics", "kw": ["chair", "desk"]})),
            )
            .unwrap();
        assert_eq!(inv.method, "keyword.seo.metrics");
        assert!(inv
            .params
            .contains(&("kw".into(), r#"["chair","desk"]"#.into())));

        let err = registry
            .resolve(
                "sistrix_keyword",
                &args(json!({"report": "rankings", "kw": ["chair", "desk"]})),
            )
            .unwrap_err();
        assert!(err.contains("metrics"), "unexpected error: {err}");

        let err = registry
            .resolve(
                "sistrix_keyword",
                &args(json!({"report": "metrics", "kw": []})),
            )
            .unwrap_err();
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }

    #[test]
    fn regex_filters_pass_through() {
        let registry = Registry::new(None);
        let inv = registry
            .resolve(
                "sistrix_domain_rankings",
                &args(json!({
                    "target": "example.com",
                    "regex_keyword": "^buy .*",
                    "regex_url": "/shop/"
                })),
            )
            .unwrap();
        assert!(inv
            .params
            .contains(&("regex_keyword".into(), "^buy .*".into())));
        assert!(inv.params.contains(&("regex_url".into(), "/shop/".into())));
    }

    #[test]
    fn every_tool_has_title_and_annotations() {
        let registry = Registry::new(None);
        for tool in registry.mcp_tools() {
            let title = tool.title.as_deref().unwrap_or_default();
            assert!(!title.is_empty(), "tool '{}' missing title", tool.name);

            let annotations = tool
                .annotations
                .as_ref()
                .unwrap_or_else(|| panic!("tool '{}' missing annotations", tool.name));
            let read_only = annotations.read_only_hint;
            if tool.name == "sistrix_api" {
                assert_eq!(read_only, Some(false), "raw tool can reach write methods");
                assert_eq!(annotations.destructive_hint, Some(false));
            } else {
                assert_eq!(read_only, Some(true), "'{}' is a query tool", tool.name);
            }
            assert_eq!(annotations.open_world_hint, Some(true));
        }
    }

    #[test]
    fn schema_has_target_and_scope_for_domain_tools() {
        let registry = Registry::new(None);
        let tool = registry
            .mcp_tools()
            .into_iter()
            .find(|t| t.name == "sistrix_visibility")
            .unwrap();
        let props = tool.input_schema.get("properties").unwrap();
        assert!(props.get("target").is_some());
        let scope_enum = props.get("scope").and_then(|s| s.get("enum")).unwrap();
        assert!(scope_enum.as_array().unwrap().contains(&json!("url")));
        let required = tool.input_schema.get("required").unwrap();
        assert!(required.as_array().unwrap().contains(&json!("target")));
    }

    #[test]
    fn schema_select_arg_gets_enum_from_cases() {
        let registry = Registry::new(None);
        let tool = registry
            .mcp_tools()
            .into_iter()
            .find(|t| t.name == "sistrix_keyword")
            .unwrap();
        let report = tool
            .input_schema
            .get("properties")
            .and_then(|p| p.get("report"))
            .unwrap();
        let choices = report.get("enum").unwrap().as_array().unwrap();
        assert!(choices.contains(&json!("metrics")));
        assert!(choices.contains(&json!("questions")));
    }

    #[test]
    fn schema_mentions_default_country() {
        let registry = Registry::new(Some("de".into()));
        let tool = registry
            .mcp_tools()
            .into_iter()
            .find(|t| t.name == "sistrix_domain_overview")
            .unwrap();
        let country = tool
            .input_schema
            .get("properties")
            .and_then(|p| p.get("country"))
            .unwrap();
        assert!(country
            .get("description")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("'de'"));
    }
}
