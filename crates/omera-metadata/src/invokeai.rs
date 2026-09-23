//! InvokeAI metadata parser from `sd-metadata`, `invokeai_metadata`, or `dream` chunks.

use omera_domain::{ExtractedMetadata, MetadataFormat};
use serde_json::Value;

/// Attempt to parse InvokeAI metadata from JSON string (`sd-metadata` or `invokeai_metadata`).
pub fn parse_invokeai(json_str: &str) -> Option<ExtractedMetadata> {
    let root: Value = serde_json::from_str(json_str).ok()?;
    let obj = root.as_object()?;

    // Can have "image" object inside (legacy invokeai) or direct fields
    let map = if let Some(img_obj) = obj.get("image").and_then(|v| v.as_object()) {
        img_obj
    } else {
        obj
    };

    let prompt = map
        .get("positive_prompt")
        .or_else(|| map.get("prompt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let negative_prompt = map
        .get("negative_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let steps = map.get("steps").and_then(|v| v.as_u64()).map(|s| s as u32);
    let cfg_scale = map
        .get("cfg_scale")
        .or_else(|| map.get("cfg"))
        .and_then(|v| v.as_f64());

    let seed = map.get("seed").map(|v| match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    });

    let sampler = map
        .get("scheduler")
        .or_else(|| map.get("sampler"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let model_name = map
        .get("model")
        .and_then(|v| {
            if let Some(m_obj) = v.as_object() {
                m_obj
                    .get("model_name")
                    .or_else(|| m_obj.get("name"))
                    .and_then(|n| n.as_str())
            } else {
                v.as_str()
            }
        })
        .map(|s| s.to_string());

    let width = map.get("width").and_then(|v| v.as_u64()).map(|w| w as u32);
    let height = map.get("height").and_then(|v| v.as_u64()).map(|h| h as u32);

    if prompt.is_none() && steps.is_none() && model_name.is_none() {
        return None;
    }

    Some(ExtractedMetadata {
        format: MetadataFormat::InvokeAI,
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
    fn parses_invokeai_json() {
        let json = r#"{
            "model": { "model_name": "sd_xl_base_1.0" },
            "positive_prompt": "cinematic shot of an astronaut on Mars",
            "negative_prompt": "cartoon, illustration",
            "steps": 30,
            "cfg_scale": 7.5,
            "seed": 923841,
            "scheduler": "euler_a",
            "width": 1024,
            "height": 1024
        }"#;
        let meta = parse_invokeai(json).expect("parsed");
        assert_eq!(meta.format, MetadataFormat::InvokeAI);
        assert_eq!(
            meta.prompt.as_deref(),
            Some("cinematic shot of an astronaut on Mars")
        );
        assert_eq!(
            meta.negative_prompt.as_deref(),
            Some("cartoon, illustration")
        );
        assert_eq!(meta.steps, Some(30));
        assert_eq!(meta.cfg_scale, Some(7.5));
        assert_eq!(meta.sampler.as_deref(), Some("euler_a"));
        assert_eq!(meta.seed.as_deref(), Some("923841"));
        assert_eq!(meta.model_name.as_deref(), Some("sd_xl_base_1.0"));
        assert_eq!(meta.width, Some(1024));
        assert_eq!(meta.height, Some(1024));
    }
}
