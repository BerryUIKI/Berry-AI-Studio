//! EasyDiffusion and Stable Swarm metadata parsers.

use berry_domain::{ExtractedMetadata, MetadataFormat};
use serde_json::Value;

/// Attempt to parse EasyDiffusion metadata JSON.
pub fn parse_easydiffusion(json_str: &str) -> Option<ExtractedMetadata> {
    let root: Value = serde_json::from_str(json_str).ok()?;
    let obj = root.as_object()?;

    if !obj.contains_key("use_stable_diffusion_model") && !obj.contains_key("num_inference_steps") {
        return None;
    }

    let prompt = obj
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let negative_prompt = obj
        .get("negative_prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let steps = obj
        .get("num_inference_steps")
        .and_then(|v| v.as_u64())
        .map(|s| s as u32);
    let cfg_scale = obj.get("guidance_scale").and_then(|v| v.as_f64());
    let seed = obj.get("seed").map(|v| match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    });
    let sampler = obj
        .get("sampler_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let model_name = obj
        .get("use_stable_diffusion_model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let width = obj.get("width").and_then(|v| v.as_u64()).map(|w| w as u32);
    let height = obj.get("height").and_then(|v| v.as_u64()).map(|h| h as u32);

    Some(ExtractedMetadata {
        format: MetadataFormat::EasyDiffusion,
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

/// Attempt to parse Stable Swarm metadata (`sui_image_params`).
pub fn parse_stableswarm(json_str: &str) -> Option<ExtractedMetadata> {
    let root: Value = serde_json::from_str(json_str).ok()?;
    let obj = root
        .get("sui_image_params")
        .and_then(|v| v.as_object())
        .or_else(|| root.as_object())?;

    if !obj.contains_key("sui_image_params")
        && !obj.contains_key("swarm_version")
        && !obj.contains_key("cfgscale")
    {
        return None;
    }

    let prompt = obj
        .get("prompt")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let negative_prompt = obj
        .get("negativeprompt")
        .or_else(|| obj.get("negative_prompt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let steps = obj.get("steps").and_then(|v| v.as_u64()).map(|s| s as u32);
    let cfg_scale = obj
        .get("cfgscale")
        .or_else(|| obj.get("cfg"))
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
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let width = obj.get("width").and_then(|v| v.as_u64()).map(|w| w as u32);
    let height = obj.get("height").and_then(|v| v.as_u64()).map(|h| h as u32);

    Some(ExtractedMetadata {
        format: MetadataFormat::StableSwarm,
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
    fn parses_easydiffusion_json() {
        let json = r#"{"prompt": "retro synthwave car", "negative_prompt": "blurry", "num_inference_steps": 25, "guidance_scale": 8.0, "seed": 112233, "sampler_name": "euler_a", "use_stable_diffusion_model": "sd_v1-5"}"#;
        let meta = parse_easydiffusion(json).expect("parsed");
        assert_eq!(meta.format, MetadataFormat::EasyDiffusion);
        assert_eq!(meta.prompt.as_deref(), Some("retro synthwave car"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(25));
        assert_eq!(meta.cfg_scale, Some(8.0));
        assert_eq!(meta.sampler.as_deref(), Some("euler_a"));
        assert_eq!(meta.model_name.as_deref(), Some("sd_v1-5"));
    }
}
