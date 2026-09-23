//! Query string parser converting user search input into structured `SearchCriteria`.

use crate::SearchCriteria;

/// Parse a human-entered search query into structured `SearchCriteria`.
///
/// Supports:
/// - Key-value tokens: `prompt:...`, `neg:...`, `model:...`, `hash:...`, `sampler:...`
/// - Numeric ranges: `steps:20..40`, `cfg:5.0..8.5`, `rating:7..10`
/// - Comparison operators: `steps:>=20`, `rating:>=8`, `cfg:<10`
/// - Quoted values: `model:"dreamshaper xl"`, `prompt:"neon cityscape"`
/// - Bare words: combined into broad `text` search across prompt, model, and path.
pub fn parse_query(input: &str) -> SearchCriteria {
    let mut criteria = SearchCriteria::default();
    let raw_tokens = tokenize(input);

    // Stitch tokens where a key ends with ':' and the value was separated by a space
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < raw_tokens.len() {
        let token = &raw_tokens[i];
        if token.ends_with(':') && i + 1 < raw_tokens.len() {
            tokens.push(format!("{}{}", token, raw_tokens[i + 1]));
            i += 2;
        } else {
            tokens.push(token.clone());
            i += 1;
        }
    }

    let mut bare_terms = Vec::new();

    for token in tokens {
        if let Some((key, val)) = token.split_once(':') {
            let key = key.trim().to_ascii_lowercase();
            let val = val.trim();
            if val.is_empty() {
                continue;
            }

            match key.as_str() {
                "prompt" => {
                    criteria.prompt = Some(val.to_string());
                }
                "neg" | "negative" | "negative_prompt" => {
                    criteria.negative_prompt = Some(val.to_string());
                }
                "model" | "model_name" => {
                    criteria.model_name = Some(val.to_string());
                }
                "hash" | "model_hash" => {
                    criteria.model_hash = Some(val.to_string());
                }
                "sampler" => {
                    criteria.sampler = Some(val.to_string());
                }
                "steps" => {
                    if let Some((min, max)) = parse_u32_range(val) {
                        criteria.min_steps = min;
                        criteria.max_steps = max;
                    }
                }
                "cfg" | "cfg_scale" => {
                    if let Some((min, max)) = parse_f64_range(val) {
                        criteria.min_cfg = min;
                        criteria.max_cfg = max;
                    }
                }
                "rating" => {
                    if let Some((min, max)) = parse_u8_range(val) {
                        criteria.min_rating = min;
                        criteria.max_rating = max;
                    }
                }
                "aesthetic" | "aesthetic_score" => {
                    if let Some((min, max)) = parse_f64_range(val) {
                        criteria.min_aesthetic = min;
                        criteria.max_aesthetic = max;
                    }
                }
                "fav" | "favorite" => match val.to_ascii_lowercase().as_str() {
                    "true" | "yes" | "1" => criteria.is_favorite = Some(true),
                    "false" | "no" | "0" => criteria.is_favorite = Some(false),
                    _ => {}
                },
                "nsfw" => match val.to_ascii_lowercase().as_str() {
                    "true" | "yes" | "1" => criteria.is_nsfw = Some(true),
                    "false" | "no" | "0" => criteria.is_nsfw = Some(false),
                    _ => {}
                },
                "is" => match val.to_ascii_lowercase().as_str() {
                    "fav" | "favorite" => criteria.is_favorite = Some(true),
                    "nsfw" => criteria.is_nsfw = Some(true),
                    "sfw" => criteria.is_nsfw = Some(false),
                    _ => {}
                },
                "album" | "album_id" => {
                    if let Ok(id) = val.parse::<i64>() {
                        criteria.album_id = Some(id);
                    }
                }
                "tag" | "tag_id" => {
                    if let Ok(id) = val.parse::<i64>() {
                        criteria.tag_id = Some(id);
                    }
                }
                _ => {
                    // Unknown key: treat the whole token as a search term
                    bare_terms.push(token);
                }
            }
        } else {
            bare_terms.push(token);
        }
    }

    if !bare_terms.is_empty() {
        criteria.text = Some(bare_terms.join(" "));
    }

    criteria
}

impl SearchCriteria {
    /// Convenience constructor parsing a search query string.
    pub fn from_query(query: &str) -> Self {
        parse_query(query)
    }
}

/// Tokenize input respecting single and double quotes.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in input.chars() {
        if in_quotes {
            if ch == quote_char {
                in_quotes = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_quotes = true;
            quote_char = ch;
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_u32_range(val: &str) -> Option<(Option<u32>, Option<u32>)> {
    if let Some((start, end)) = val.split_once("..") {
        let min = start.trim().parse::<u32>().ok();
        let max = end.trim().parse::<u32>().ok();
        return Some((min, max));
    }
    if let Some((start, end)) = val.split_once('-') {
        if !start.is_empty() && !end.is_empty() {
            let min = start.trim().parse::<u32>().ok();
            let max = end.trim().parse::<u32>().ok();
            return Some((min, max));
        }
    }
    if let Some(s) = val.strip_prefix(">=") {
        return Some((s.trim().parse::<u32>().ok(), None));
    }
    if let Some(s) = val.strip_prefix('>') {
        return Some((
            s.trim().parse::<u32>().ok().map(|n| n.saturating_add(1)),
            None,
        ));
    }
    if let Some(s) = val.strip_prefix("<=") {
        return Some((None, s.trim().parse::<u32>().ok()));
    }
    if let Some(s) = val.strip_prefix('<') {
        return Some((
            None,
            s.trim().parse::<u32>().ok().map(|n| n.saturating_sub(1)),
        ));
    }
    if let Some(s) = val.strip_prefix('=') {
        let n = s.trim().parse::<u32>().ok();
        return Some((n, n));
    }
    let n = val.parse::<u32>().ok();
    Some((n, n))
}

fn parse_u8_range(val: &str) -> Option<(Option<u8>, Option<u8>)> {
    if let Some((start, end)) = val.split_once("..") {
        let min = start.trim().parse::<u8>().ok();
        let max = end.trim().parse::<u8>().ok();
        return Some((min, max));
    }
    if let Some((start, end)) = val.split_once('-') {
        if !start.is_empty() && !end.is_empty() {
            let min = start.trim().parse::<u8>().ok();
            let max = end.trim().parse::<u8>().ok();
            return Some((min, max));
        }
    }
    if let Some(s) = val.strip_prefix(">=") {
        return Some((s.trim().parse::<u8>().ok(), None));
    }
    if let Some(s) = val.strip_prefix('>') {
        return Some((
            s.trim().parse::<u8>().ok().map(|n| n.saturating_add(1)),
            None,
        ));
    }
    if let Some(s) = val.strip_prefix("<=") {
        return Some((None, s.trim().parse::<u8>().ok()));
    }
    if let Some(s) = val.strip_prefix('<') {
        return Some((
            None,
            s.trim().parse::<u8>().ok().map(|n| n.saturating_sub(1)),
        ));
    }
    if let Some(s) = val.strip_prefix('=') {
        let n = s.trim().parse::<u8>().ok();
        return Some((n, n));
    }
    let n = val.parse::<u8>().ok();
    Some((n, n))
}

fn parse_f64_range(val: &str) -> Option<(Option<f64>, Option<f64>)> {
    if let Some((start, end)) = val.split_once("..") {
        let min = start.trim().parse::<f64>().ok();
        let max = end.trim().parse::<f64>().ok();
        return Some((min, max));
    }
    if let Some(s) = val.strip_prefix(">=") {
        return Some((s.trim().parse::<f64>().ok(), None));
    }
    if let Some(s) = val.strip_prefix('>') {
        return Some((s.trim().parse::<f64>().ok(), None));
    }
    if let Some(s) = val.strip_prefix("<=") {
        return Some((None, s.trim().parse::<f64>().ok()));
    }
    if let Some(s) = val.strip_prefix('<') {
        return Some((None, s.trim().parse::<f64>().ok()));
    }
    if let Some(s) = val.strip_prefix('=') {
        let n = s.trim().parse::<f64>().ok();
        return Some((n, n));
    }
    let n = val.parse::<f64>().ok();
    Some((n, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bare_text() {
        let criteria = parse_query("cyberpunk cat 8k");
        assert_eq!(criteria.text, Some("cyberpunk cat 8k".to_string()));
        assert!(criteria.prompt.is_none());
        assert!(criteria.model_name.is_none());
    }

    #[test]
    fn test_key_values_and_quotes() {
        let criteria = parse_query(
            r#"model:"dreamshaper xl" prompt:"neon city" neg:blurry sampler:"Euler a""#,
        );
        assert_eq!(criteria.model_name, Some("dreamshaper xl".to_string()));
        assert_eq!(criteria.prompt, Some("neon city".to_string()));
        assert_eq!(criteria.negative_prompt, Some("blurry".to_string()));
        assert_eq!(criteria.sampler, Some("Euler a".to_string()));
        assert!(criteria.text.is_none());
    }

    #[test]
    fn test_space_after_colon() {
        let criteria = parse_query(r#"model: "dreamshaper xl" prompt: "neon city""#);
        assert_eq!(criteria.model_name, Some("dreamshaper xl".to_string()));
        assert_eq!(criteria.prompt, Some("neon city".to_string()));
    }

    #[test]
    fn test_numeric_ranges() {
        let criteria = parse_query("steps:20..40 cfg:5.0..8.5 rating:8");
        assert_eq!(criteria.min_steps, Some(20));
        assert_eq!(criteria.max_steps, Some(40));
        assert_eq!(criteria.min_cfg, Some(5.0));
        assert_eq!(criteria.max_cfg, Some(8.5));
        assert_eq!(criteria.min_rating, Some(8));
        assert_eq!(criteria.max_rating, Some(8));
    }

    #[test]
    fn test_comparison_operators() {
        let criteria = parse_query("steps:>=25 rating:>=7 cfg:<12.0");
        assert_eq!(criteria.min_steps, Some(25));
        assert_eq!(criteria.max_steps, None);
        assert_eq!(criteria.min_rating, Some(7));
        assert_eq!(criteria.max_rating, None);
        assert_eq!(criteria.min_cfg, None);
        assert_eq!(criteria.max_cfg, Some(12.0));
    }

    #[test]
    fn test_mixed_query() {
        let criteria = parse_query("anime girl model:SDXL rating:>=8");
        assert_eq!(criteria.text, Some("anime girl".to_string()));
        assert_eq!(criteria.model_name, Some("SDXL".to_string()));
        assert_eq!(criteria.min_rating, Some(8));
    }

    #[test]
    fn test_organization_tokens() {
        let c1 = parse_query("fav:true nsfw:false album:42 tag:7");
        assert_eq!(c1.is_favorite, Some(true));
        assert_eq!(c1.is_nsfw, Some(false));
        assert_eq!(c1.album_id, Some(42));
        assert_eq!(c1.tag_id, Some(7));

        let c2 = parse_query("is:fav is:nsfw");
        assert_eq!(c2.is_favorite, Some(true));
        assert_eq!(c2.is_nsfw, Some(true));

        let c3 = parse_query("is:sfw");
        assert_eq!(c3.is_nsfw, Some(false));
    }
}
