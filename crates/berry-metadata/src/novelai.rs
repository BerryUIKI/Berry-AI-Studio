//! NovelAI metadata parser from `Comment` (JSON) or `Description` PNG text chunks.

use berry_domain::{ExtractedMetadata, MetadataFormat};
use serde_json::Value;

/// Attempt to parse NovelAI metadata from JSON string (typically inside `Comment` chunk).
pub fn parse_novelai(json_str: &str, description: Option<&str>) -> Option<ExtractedMetadata> {
    let root: Value = serde_json::from_str(json_str).ok()?;
    let obj = root.as_object()?;

    // Must look like NovelAI comment (contains prompt or uc or scale or sampler)
    if !obj.contains_key("prompt") && !obj.contains_key("uc") && !obj.contains_key("scale") {
        return None;
    }

    let prompt = obj
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| description.map(|d| d.to_string()));

    let negative_prompt = obj
        .get("uc")
        .or_else(|| obj.get("undesired_content"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let steps = obj.get("steps").and_then(|v| v.as_u64()).map(|s| s as u32);
    let cfg_scale = obj
        .get("scale")
        .or_else(|| obj.get("cfg_scale"))
        .and_then(|v| v.as_f64());

    let seed = obj.get("seed").map(|v| match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    });

    let sampler = obj
        .get("sampler")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let model_name = obj
        .get("model")
        .or_else(|| obj.get("Source"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let width = obj.get("width").and_then(|v| v.as_u64()).map(|w| w as u32);
    let height = obj.get("height").and_then(|v| v.as_u64()).map(|h| h as u32);

    Some(ExtractedMetadata {
        format: MetadataFormat::NovelAI,
        parameters: Some(json_str.to_string()),
        raw: Some(json_str.to_string()),
        prompt,
        negative_prompt,
        width,
        height,
        seed,
        steps,
        cfg_scale,
        sampler,
        model_name,
        model_hash: None,
        duration_seconds: None,
        fps: None,
        video_codec: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_novelai_comment_json() {
        let json = r#"{"prompt": "masterpiece, 1girl, forest", "uc": "lowres, bad hands", "steps": 28, "scale": 6.0, "seed": 4028491823, "sampler": "k_euler"}"#;
        let meta = parse_novelai(json, None).expect("parsed");
        assert_eq!(meta.format, MetadataFormat::NovelAI);
        assert_eq!(meta.prompt.as_deref(), Some("masterpiece, 1girl, forest"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("lowres, bad hands"));
        assert_eq!(meta.steps, Some(28));
        assert_eq!(meta.cfg_scale, Some(6.0));
        assert_eq!(meta.sampler.as_deref(), Some("k_euler"));
        assert_eq!(meta.seed.as_deref(), Some("4028491823"));
    }
}
