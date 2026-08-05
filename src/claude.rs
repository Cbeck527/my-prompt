use serde::Deserialize;

/// Session information received from Claude Code's status-line input.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ClaudeSession {
    pub(crate) model_name: String,
    pub(crate) context_used: u64,
    pub(crate) context_total: u64,
    pub(crate) percentage: u8,
}

#[derive(Deserialize)]
struct ClaudeInput {
    model: ClaudeModel,
    context_window: ClaudeContextWindow,
}

#[derive(Deserialize)]
struct ClaudeModel {
    display_name: String,
}

#[derive(Deserialize)]
struct ClaudeContextWindow {
    context_window_size: u64,
    used_percentage: Option<f64>,
    current_usage: Option<ClaudeContextWindowCurrentUsage>,
}

#[expect(
    clippy::struct_field_names,
    reason = "the field names intentionally mirror Claude Code's JSON schema"
)]
#[derive(Deserialize)]
struct ClaudeContextWindowCurrentUsage {
    input_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
}

pub(crate) fn parse_json(input: &str) -> Option<ClaudeSession> {
    let parsed: ClaudeInput = serde_json::from_str(input).ok()?;

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is rounded and clamped to 0 through 100 before conversion"
    )]
    let percentage = parsed
        .context_window
        .used_percentage
        .map_or(0, |value| value.round().clamp(0.0, 100.0) as u8);

    // Claude's context usage excludes output tokens. Before the first API call,
    // current_usage is absent and therefore represents zero usage.
    let context_used = match parsed.context_window.current_usage.as_ref() {
        Some(usage) => usage
            .input_tokens
            .checked_add(usage.cache_creation_input_tokens)?
            .checked_add(usage.cache_read_input_tokens)?,
        None => 0,
    };

    Some(ClaudeSession {
        model_name: parsed.model.display_name,
        context_used,
        context_total: parsed.context_window.context_window_size,
        percentage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(used_percentage: Option<f64>) -> String {
        serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": used_percentage,
                "current_usage": {
                    "input_tokens": 8_500,
                    "output_tokens": 99_999,
                    "cache_creation_input_tokens": 5_000,
                    "cache_read_input_tokens": 2_000
                }
            }
        })
        .to_string()
    }

    #[test]
    fn valid_input_maps_to_a_session() {
        assert_eq!(
            parse_json(&input(Some(8.0))),
            Some(ClaudeSession {
                model_name: "Opus".to_owned(),
                context_used: 15_500,
                context_total: 200_000,
                percentage: 8,
            })
        );
    }

    #[test]
    fn percentage_is_rounded_and_clamped() {
        assert_eq!(
            parse_json(&input(Some(8.5))).map(|session| session.percentage),
            Some(9)
        );
        assert_eq!(
            parse_json(&input(Some(101.0))).map(|session| session.percentage),
            Some(100)
        );
        assert_eq!(
            parse_json(&input(Some(-1.0))).map(|session| session.percentage),
            Some(0)
        );
    }

    #[test]
    fn missing_usage_represents_an_early_session() {
        let input = serde_json::json!({
            "model": { "display_name": "Sonnet" },
            "context_window": { "context_window_size": 200_000 }
        })
        .to_string();

        assert_eq!(
            parse_json(&input),
            Some(ClaudeSession {
                model_name: "Sonnet".to_owned(),
                context_used: 0,
                context_total: 200_000,
                percentage: 0,
            })
        );
    }

    #[test]
    fn overflowing_context_usage_is_rejected() {
        let input = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "current_usage": {
                    "input_tokens": u64::MAX,
                    "cache_creation_input_tokens": 1,
                    "cache_read_input_tokens": 0
                }
            }
        })
        .to_string();

        assert!(parse_json(&input).is_none());
    }

    #[test]
    fn malformed_or_incomplete_input_is_rejected() {
        assert!(parse_json("").is_none());
        assert!(parse_json("{not json}").is_none());
        assert!(parse_json(r#"{"model":{"display_name":"Opus"}}"#).is_none());
    }
}
