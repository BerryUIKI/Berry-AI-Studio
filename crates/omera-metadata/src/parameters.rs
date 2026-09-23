//! Parsing of A1111 / SD.Next PNGInfo `parameters` strings.
//!
//! A1111 stores generation settings as a `tEXt` chunk whose text is
//! newline-separated: the positive prompt, an optional `Negative prompt:` line,
//! and a final line of comma-separated `Key: value` settings:
//!
//! ```text
//! (masterpiece:1.2), a portrait of a woman
//!
//! Negative prompt: lowres, bad hands
//! Steps: 28, Sampler: DPM++ 2M Karras, CFG scale: 7, Seed: 1716021952,
//! Size: 832x1216, Model hash: abc123, Model: realisticVision.safetensors
//! ```
//!
//! The settings line can also share a line with the prompt/negative prompt when
//! the image was saved without line breaks. Parsing is best-effort: unknown or
//! malformed keys are skipped rather than failing the whole parse.

/// Parameter keys understood by the parser (as they appear in the settings line).
const PARAM_PREFIXES: &[&str] = &[
    "Steps:",
    "Sampler:",
    "CFG scale:",
    "Seed:",
    "Size:",
    "Model hash:",
    "Model:",
];

/// The parsed result of an A1111 `parameters` string. Optional fields are
/// `None` when the source string did not provide them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedParameters {
    /// Positive prompt.
    pub prompt: Option<String>,
    /// Negative prompt, if any.
    pub negative_prompt: Option<String>,
    /// Number of sampling steps.
    pub steps: Option<u32>,
    /// Sampler name.
    pub sampler: Option<String>,
    /// CFG scale.
    pub cfg_scale: Option<f64>,
    /// Seed, kept as a string (seeds can exceed 32 bits).
    pub seed: Option<String>,
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
    /// Checkpoint model name.
    pub model_name: Option<String>,
    /// Checkpoint model hash.
    pub model_hash: Option<String>,
}

/// Parse an A1111/SD.Next `parameters` string into structured fields.
pub fn parse_parameters(text: &str) -> ParsedParameters {
    let normalized = text.replace("\r\n", "\n").trim_end().to_string();
    let lines: Vec<&str> = normalized.split('\n').collect();

    // The settings line is the last line containing a known parameter key.
    let settings_index = lines
        .iter()
        .rposition(|line| PARAM_PREFIXES.iter().any(|p| line.contains(p)));

    match settings_index {
        Some(index) => {
            let head = lines[..index].join("\n");
            let settings = lines[index];
            let (prompt, negative_prompt) = split_prompt_and_negative(&head, settings);
            let settings_text = &settings[first_param_key_pos(settings).unwrap_or(0)..];
            ParsedParameters {
                prompt,
                negative_prompt,
                ..parse_settings(settings_text)
            }
        }
        // No settings line: the whole string is the prompt.
        None => ParsedParameters {
            prompt: non_empty(normalized.trim()),
            ..ParsedParameters::default()
        },
    }
}

/// Index of the first parameter key in `line`, if any.
fn first_param_key_pos(line: &str) -> Option<usize> {
    PARAM_PREFIXES.iter().filter_map(|p| line.find(p)).min()
}

/// Split the region before the settings line into prompt and negative prompt.
///
/// `head` holds every line before the settings line; `settings` may itself
/// carry the prompt/negative prompt inline when they share the settings line.
fn split_prompt_and_negative(head: &str, settings: &str) -> (Option<String>, Option<String>) {
    // In the single-line form the prompt and negative prompt precede the first
    // parameter key on the settings line itself; otherwise they live in `head`.
    let before_settings = if head.is_empty() {
        let end = first_param_key_pos(settings).unwrap_or(settings.len());
        &settings[..end]
    } else {
        head
    };

    match before_settings.find("Negative prompt:") {
        Some(pos) => {
            let prompt = non_empty(before_settings[..pos].trim());
            let negative = non_empty(before_settings[pos + "Negative prompt:".len()..].trim());
            (prompt, negative)
        }
        None => (non_empty(before_settings.trim()), None),
    }
}

/// Parse the comma-separated `Key: value` settings line.
fn parse_settings(settings: &str) -> ParsedParameters {
    let mut parsed = ParsedParameters::default();
    for segment in settings.split(',') {
        let Some((key, value)) = segment.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key {
            "Steps" => parsed.steps = value.parse().ok(),
            "Sampler" => parsed.sampler = Some(value.to_string()),
            "CFG scale" => parsed.cfg_scale = value.parse().ok(),
            "Seed" => parsed.seed = Some(value.to_string()),
            "Size" => {
                if let Some((width, height)) = parse_size(value) {
                    parsed.width = Some(width);
                    parsed.height = Some(height);
                }
            }
            "Model hash" => parsed.model_hash = Some(value.to_string()),
            "Model" => parsed.model_name = Some(value.to_string()),
            _ => {}
        }
    }
    parsed
}

/// Parse a `WxH` size value (case-insensitive `x`, also accepts `×`).
fn parse_size(value: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = value.split(['x', 'X', '×']).collect();
    if parts.len() != 2 {
        return None;
    }
    let width = parts[0].trim().parse().ok()?;
    let height = parts[1].trim().parse().ok()?;
    Some((width, height))
}

/// Trim and return `None` for blank strings.
fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_a1111_parameters() {
        let text = "(masterpiece, best quality:1.2), a portrait of a woman in a field\n\
                     Negative prompt: lowres, bad hands, watermark\n\
                     Steps: 28, Sampler: DPM++ 2M Karras, CFG scale: 7, Seed: 1716021952, \
                     Size: 832x1216, Model hash: 5459434d6a5c5f46, \
                     Model: realisticVisionV51_v51VAE.safetensors, Denoising strength: 0.5";
        let p = parse_parameters(text);
        assert_eq!(
            p.prompt.as_deref(),
            Some("(masterpiece, best quality:1.2), a portrait of a woman in a field")
        );
        assert_eq!(
            p.negative_prompt.as_deref(),
            Some("lowres, bad hands, watermark")
        );
        assert_eq!(p.steps, Some(28));
        assert_eq!(p.sampler.as_deref(), Some("DPM++ 2M Karras"));
        assert_eq!(p.cfg_scale, Some(7.0));
        assert_eq!(p.seed.as_deref(), Some("1716021952"));
        assert_eq!(p.width, Some(832));
        assert_eq!(p.height, Some(1216));
        assert_eq!(
            p.model_name.as_deref(),
            Some("realisticVisionV51_v51VAE.safetensors")
        );
        assert_eq!(p.model_hash.as_deref(), Some("5459434d6a5c5f46"));
    }

    #[test]
    fn parses_single_line_parameters() {
        let text = "a cat on a couch Negative prompt: blurry, low quality \
                    Steps: 20, Sampler: Euler a, CFG scale: 6, Seed: 42, Size: 512x512, \
                    Model hash: abc, Model: model.safetensors";
        let p = parse_parameters(text);
        assert_eq!(p.prompt.as_deref(), Some("a cat on a couch"));
        assert_eq!(p.negative_prompt.as_deref(), Some("blurry, low quality"));
        assert_eq!(p.steps, Some(20));
        assert_eq!(p.sampler.as_deref(), Some("Euler a"));
        assert_eq!(p.width, Some(512));
        assert_eq!(p.height, Some(512));
    }

    #[test]
    fn handles_missing_negative_prompt() {
        let text = "a landscape\n\
                    Steps: 30, Sampler: DPM++ SDE Karras, CFG scale: 5, Seed: 7, Size: 768x768";
        let p = parse_parameters(text);
        assert_eq!(p.prompt.as_deref(), Some("a landscape"));
        assert_eq!(p.negative_prompt, None);
        assert_eq!(p.steps, Some(30));
        assert_eq!(p.width, Some(768));
        assert_eq!(p.height, Some(768));
    }

    #[test]
    fn prompt_only_has_no_parameters() {
        let p = parse_parameters("just a prompt with no parameters");
        assert_eq!(
            p.prompt.as_deref(),
            Some("just a prompt with no parameters")
        );
        assert_eq!(p.negative_prompt, None);
        assert_eq!(p.steps, None);
        assert_eq!(p.sampler, None);
    }

    #[test]
    fn parses_multiline_prompt() {
        let text =
            "line one\nline two\nNegative prompt: nope\nSteps: 10, Sampler: Euler, Size: 512x384";
        let p = parse_parameters(text);
        assert_eq!(p.prompt.as_deref(), Some("line one\nline two"));
        assert_eq!(p.negative_prompt.as_deref(), Some("nope"));
    }

    #[test]
    fn handles_crlf_line_endings() {
        let text = "a robot\r\nNegative prompt: blurry\r\nSteps: 5, Sampler: Euler, Size: 512x512";
        let p = parse_parameters(text);
        assert_eq!(p.prompt.as_deref(), Some("a robot"));
        assert_eq!(p.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(p.steps, Some(5));
    }

    #[test]
    fn ignores_unknown_keys() {
        let text = "a prompt\nSteps: 20, Sampler: Euler, Denoising strength: 0.4, Clip skip: 2, Size: 512x512";
        let p = parse_parameters(text);
        assert_eq!(p.steps, Some(20));
        assert_eq!(p.sampler.as_deref(), Some("Euler"));
        assert_eq!(p.width, Some(512));
    }

    #[test]
    fn parses_size_variants() {
        assert_eq!(parse_size("512x768"), Some((512, 768)));
        assert_eq!(parse_size("1024 X 1024"), Some((1024, 1024)));
        assert_eq!(parse_size("832x1216"), Some((832, 1216)));
        assert_eq!(parse_size("nope"), None);
    }

    #[test]
    fn blank_negative_prompt_is_none() {
        let text = "a prompt\nNegative prompt: \nSteps: 20, Sampler: Euler";
        let p = parse_parameters(text);
        assert_eq!(p.negative_prompt, None);
        assert_eq!(p.prompt.as_deref(), Some("a prompt"));
    }
}
