//! Guided prompt workflows (MCP `prompts/list` + `prompts/get`).
//!
//! Clients surface these as one-click commands (Claude shows them as
//! slash-commands). Each prompt walks the model through a proven, credit-aware
//! analysis sequence over the curated tools — so a user gets a complete report
//! without knowing which of the 17 tools to combine in which order.

use rmcp::model::{GetPromptResult, Prompt, PromptArgument, PromptMessage, PromptMessageRole};
use serde_json::{Map, Value};

/// (name, description, required)
type ArgSpec = (&'static str, &'static str, bool);

struct PromptSpec {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    arguments: &'static [ArgSpec],
}

const ARG_COUNTRY: ArgSpec = (
    "country",
    "Country index to analyze, e.g. 'de', 'at', 'us'. Defaults to the account's default country.",
    false,
);

const PROMPTS: &[PromptSpec] = &[
    PromptSpec {
        name: "seo_health_check",
        title: "SEO health check",
        description: "Complete, credit-aware SEO status report for a domain: key figures, \
                      visibility trend, rankings, competitors, and the biggest opportunities.",
        arguments: &[
            ("target", "The domain to analyze, e.g. 'example.com'.", true),
            ARG_COUNTRY,
        ],
    },
    PromptSpec {
        name: "keyword_research",
        title: "Keyword research",
        description: "Deep-dive on one keyword: metrics, search intent, SERP features, \
                      questions people ask, and who ranks today.",
        arguments: &[("keyword", "The keyword to research.", true), ARG_COUNTRY],
    },
    PromptSpec {
        name: "competitor_comparison",
        title: "Competitor comparison",
        description: "Side-by-side SEO comparison of your domain and one competitor: \
                      visibility trends, keyword footprint, and where they win.",
        arguments: &[
            ("target", "Your domain, e.g. 'example.com'.", true),
            (
                "competitor",
                "The competitor domain to compare against.",
                true,
            ),
            ARG_COUNTRY,
        ],
    },
    PromptSpec {
        name: "ai_visibility_report",
        title: "AI visibility report",
        description: "How ChatGPT, Perplexity, and Google AI Overviews see a brand: \
                      visibility, competing entities, triggering prompts, and cited sources.",
        arguments: &[
            (
                "brand",
                "The brand or entity name to analyze, e.g. 'SISTRIX'.",
                true,
            ),
            ARG_COUNTRY,
        ],
    },
];

pub fn list() -> Vec<Prompt> {
    PROMPTS
        .iter()
        .map(|spec| Prompt {
            name: spec.name.to_string(),
            title: Some(spec.title.to_string()),
            description: Some(spec.description.to_string()),
            arguments: Some(
                spec.arguments
                    .iter()
                    .map(|(name, description, required)| PromptArgument {
                        name: (*name).to_string(),
                        title: None,
                        description: Some((*description).to_string()),
                        required: Some(*required),
                    })
                    .collect(),
            ),
            icons: None,
            meta: None,
        })
        .collect()
}

pub fn get(name: &str, args: Option<&Map<String, Value>>) -> Result<GetPromptResult, String> {
    let spec = PROMPTS
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("unknown prompt '{name}'"))?;

    let arg = |key: &str| -> Option<String> {
        args?.get(key).and_then(|v| match v {
            Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
            Value::Null => None,
            Value::String(_) => None,
            other => Some(other.to_string()),
        })
    };

    for (arg_name, _, required) in spec.arguments {
        if *required && arg(arg_name).is_none() {
            return Err(format!(
                "prompt '{name}' requires the argument '{arg_name}'"
            ));
        }
    }

    let country = arg("country");
    let country_note = match &country {
        Some(c) => format!("Use country='{c}' on every country-aware call."),
        None => "Use the account's default country (omit the country argument).".to_string(),
    };

    let text = match spec.name {
        "seo_health_check" => {
            let target = arg("target").unwrap();
            format!(
                "Run a complete SEO health check for {target} using the SISTRIX tools. \
                 {country_note}\n\n\
                 1. sistrix_credits — check the weekly budget first; if it is low, halve \
                 every limit below.\n\
                 2. sistrix_domain_overview target='{target}' — headline numbers.\n\
                 3. sistrix_visibility report='history' limit=26 — half-year trend; name the \
                 direction and any inflection points.\n\
                 4. sistrix_keyword_counts report='seo' and report='seo_top10' — keyword \
                 footprint.\n\
                 5. sistrix_competitors limit=10 — closest organic competitors.\n\
                 6. sistrix_keyword_ideas report='opportunities' limit=15 — quick wins just \
                 off page 1.\n\
                 7. sistrix_domain_structure report='top_urls' limit=10 — where the \
                 visibility comes from.\n\n\
                 If a step returns no results, note it and continue. Finish with: overall \
                 health verdict, trend summary, top 3 strengths, top 3 risks, and the 3 \
                 highest-impact next actions. Mention roughly how many credits the analysis \
                 used."
            )
        }
        "keyword_research" => {
            let keyword = arg("keyword").unwrap();
            format!(
                "Research the keyword '{keyword}' with the SISTRIX tools. {country_note}\n\n\
                 1. sistrix_keyword report='metrics' kw='{keyword}' — volume, CPC, \
                 competition, device split (note: 5 credits).\n\
                 2. sistrix_keyword report='search_intent' — Know/Visit/Website/Do mix.\n\
                 3. sistrix_keyword report='serp_features' — which SERP elements appear.\n\
                 4. sistrix_keyword report='questions' limit=10 — questions people ask.\n\
                 5. sistrix_keyword report='rankings' limit=10 — who ranks today.\n\
                 6. sistrix_keyword report='traffic_estimation' limit=10 — expected click \
                 share per position.\n\n\
                 If a step returns no results, note it and continue. Conclude with: is this \
                 keyword worth targeting, what content format fits the intent and SERP, and \
                 how hard will it be to rank."
            )
        }
        "competitor_comparison" => {
            let target = arg("target").unwrap();
            let competitor = arg("competitor").unwrap();
            format!(
                "Compare {target} against {competitor} with the SISTRIX tools. \
                 {country_note}\n\n\
                 For BOTH domains run:\n\
                 1. sistrix_domain_overview — headline numbers.\n\
                 2. sistrix_visibility report='history' limit=26 — half-year trend.\n\
                 3. sistrix_keyword_counts report='seo' and report='seo_top10'.\n\
                 4. sistrix_domain_structure report='top_urls' limit=5 — their strongest \
                 content.\n\n\
                 Then for {target} only: sistrix_keyword_ideas report='opportunities' \
                 limit=15 — where the gap can be closed.\n\n\
                 If a step returns no results, note it and continue. Present a side-by-side \
                 comparison table, name who is winning and why, and give 3 concrete moves \
                 for {target} to close (or extend) the gap."
            )
        }
        "ai_visibility_report" => {
            let brand = arg("brand").unwrap();
            format!(
                "Analyze how AI assistants see the brand '{brand}' using the SISTRIX AI \
                 tools. {country_note}\n\n\
                 1. sistrix_ai_entity report='overview' entity='{brand}' — key metrics \
                 (note: flat 10 credits).\n\
                 2. sistrix_ai_entity report='competition' limit=10 — which entities AI \
                 names alongside it.\n\
                 3. sistrix_ai_entity report='prompts' limit=15 — prompts where the brand \
                 appears in answers.\n\
                 4. sistrix_ai_entity report='sources' limit=15 — which sites AI cites \
                 about it.\n\
                 5. sistrix_ai_top report='brands' limit=20 — market context: the most \
                 AI-visible brands overall.\n\n\
                 If a step returns no results, note it and continue. Conclude with: how \
                 visible the brand is to AI models, who dominates its competitive space, \
                 which sources shape its AI image, and 3 actions to improve AI visibility."
            )
        }
        _ => unreachable!("spec was found in PROMPTS"),
    };

    Ok(GetPromptResult {
        description: Some(spec.description.to_string()),
        messages: vec![PromptMessage::new_text(PromptMessageRole::User, text)],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(json: Value) -> Map<String, Value> {
        json.as_object().unwrap().clone()
    }

    #[test]
    fn lists_all_prompts_with_arguments() {
        let prompts = list();
        assert_eq!(prompts.len(), 4);
        for prompt in &prompts {
            assert!(prompt.title.is_some());
            assert!(prompt.description.is_some());
            assert!(!prompt.arguments.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn substitutes_arguments_into_the_workflow() {
        let result = get(
            "seo_health_check",
            Some(&args(json!({"target": "example.com", "country": "de"}))),
        )
        .unwrap();
        let text = match &result.messages[0].content {
            rmcp::model::PromptMessageContent::Text { text } => text,
            other => panic!("expected text, got {other:?}"),
        };
        assert!(text.contains("example.com"));
        assert!(text.contains("country='de'"));
        assert!(text.contains("sistrix_credits"));
    }

    #[test]
    fn omitted_country_falls_back_to_account_default() {
        let result = get(
            "keyword_research",
            Some(&args(json!({"keyword": "ergonomic chair"}))),
        )
        .unwrap();
        let text = match &result.messages[0].content {
            rmcp::model::PromptMessageContent::Text { text } => text,
            other => panic!("expected text, got {other:?}"),
        };
        assert!(text.contains("ergonomic chair"));
        assert!(text.contains("account's default country"));
    }

    #[test]
    fn missing_required_argument_is_rejected() {
        let err = get(
            "competitor_comparison",
            Some(&args(json!({"target": "a.com"}))),
        )
        .unwrap_err();
        assert!(err.contains("competitor"));
    }

    #[test]
    fn unknown_prompt_is_rejected() {
        let err = get("galaxy_report", None).unwrap_err();
        assert!(err.contains("galaxy_report"));
    }

    #[test]
    fn every_prompt_renders_with_minimal_arguments() {
        for spec_name in [
            "seo_health_check",
            "keyword_research",
            "competitor_comparison",
            "ai_visibility_report",
        ] {
            let full = args(json!({
                "target": "example.com",
                "competitor": "rival.com",
                "keyword": "test",
                "brand": "TestBrand"
            }));
            let result = get(spec_name, Some(&full)).unwrap();
            assert_eq!(result.messages.len(), 1, "prompt '{spec_name}'");
        }
    }
}
