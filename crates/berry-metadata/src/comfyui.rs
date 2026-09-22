//! ComfyUI metadata parsing from node graph JSON (`prompt` or `workflow` chunks).

use berry_domain::{ExtractedMetadata, MetadataFormat};
use serde_json::Value;

/// Attempt to parse ComfyUI metadata from JSON text (found in `prompt` or `workflow` chunks).
pub fn parse_comfyui(json_str: &str) -> Option<ExtractedMetadata> {
    let root: Value = serde_json::from_str(json_str).ok()?;

    // The root might be { "prompt": { ... } } or directly { "node_id": { ... } } or { "nodes": [...] }
    let nodes_map = if let Some(prompt_obj) = root.get("prompt").and_then(|p| p.as_object()) {
        prompt_obj
    } else {
        let obj = root.as_object()?;
        if obj.contains_key("nodes") {
            // Workflow format with nodes array
            return parse_comfyui_workflow(&root, json_str);
        }
        obj
    };

    let mut prompt = None;
    let mut negative_prompt = None;
    let mut steps = None;
    let mut cfg_scale = None;
    let mut seed = None;
    let mut sampler = None;
    let mut model_name = None;
    let mut width = None;
    let mut height = None;

    // Find KSampler node to resolve positive / negative and sampling params
    for (_node_id, node) in nodes_map {
        let class_type = node
            .get("class_type")
            .and_then(|c| c.as_str())
            .unwrap_or_default();

        let is_sampler = class_type.contains("KSampler")
            || class_type == "SamplerCustom"
            || class_type.contains("Sampler")
            || class_type.ends_with("Sample");

        if is_sampler {
            if let Some(inputs) = node.get("inputs") {
                if steps.is_none() {
                    steps = inputs
                        .get("steps")
                        .or_else(|| inputs.get("num_inference_steps"))
                        .or_else(|| inputs.get("sampling_steps"))
                        .and_then(|v| v.as_u64())
                        .map(|s| s as u32);
                }
                if cfg_scale.is_none() {
                    cfg_scale = inputs
                        .get("cfg")
                        .or_else(|| inputs.get("guidance_scale"))
                        .or_else(|| inputs.get("cfg_scale"))
                        .and_then(|v| v.as_f64());
                }
                if seed.is_none() {
                    seed =
                        inputs
                            .get("seed")
                            .or_else(|| inputs.get("noise_seed"))
                            .map(|v| match v {
                                Value::Number(n) => n.to_string(),
                                Value::String(s) => s.clone(),
                                _ => v.to_string(),
                            });
                }
                if sampler.is_none() {
                    let s_name = inputs
                        .get("sampler_name")
                        .or_else(|| inputs.get("sampler"))
                        .and_then(|v| v.as_str());
                    let scheduler = inputs.get("scheduler").and_then(|v| v.as_str());
                    sampler = match (s_name, scheduler) {
                        (Some(s), Some(sch)) if !sch.is_empty() && sch != "normal" => {
                            Some(format!("{s}_{sch}"))
                        }
                        (Some(s), _) => Some(s.to_string()),
                        _ => None,
                    };
                }

                // Resolve positive prompt link
                if prompt.is_none() {
                    let pos_key = if inputs.get("positive").is_some() {
                        "positive"
                    } else if inputs.get("prompt").is_some() {
                        "prompt"
                    } else {
                        "pos"
                    };
                    if let Some(pos_link) = inputs.get(pos_key).and_then(|v| v.as_array()) {
                        if let Some(target_id) = pos_link.first().and_then(|v| v.as_str()) {
                            prompt = extract_clip_text(nodes_map, target_id);
                        }
                    }
                }

                // Resolve negative prompt link
                if negative_prompt.is_none() {
                    let neg_key = if inputs.get("negative").is_some() {
                        "negative"
                    } else {
                        "neg"
                    };
                    if let Some(neg_link) = inputs.get(neg_key).and_then(|v| v.as_array()) {
                        if let Some(target_id) = neg_link.first().and_then(|v| v.as_str()) {
                            negative_prompt = extract_clip_text(nodes_map, target_id);
                        }
                    }
                }
            }
        }

        // Model / Checkpoint loader (handles SD, Flux, Wan, Hunyuan, CogVideo, LTXV, SVD)
        let is_loader = class_type.contains("CheckpointLoader")
            || class_type.contains("UNETLoader")
            || class_type.contains("ModelLoader")
            || class_type.contains("WanVideo")
            || class_type.contains("Hunyuan")
            || class_type.contains("CogVideo")
            || class_type.contains("LTXV");

        if model_name.is_none() && is_loader {
            if let Some(inputs) = node.get("inputs") {
                model_name = inputs
                    .get("ckpt_name")
                    .or_else(|| inputs.get("unet_name"))
                    .or_else(|| inputs.get("model_name"))
                    .or_else(|| inputs.get("transformer_name"))
                    .or_else(|| inputs.get("diffusion_model"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }

        // Latent dimensions (EmptyLatentImage, WanVideoEmptyLatent, CogVideoXEmptyLatent, EmptyHunyuanLatentVideo, etc.)
        let is_latent = class_type.contains("EmptyLatent")
            || class_type.contains("LatentVideo")
            || class_type.contains("EmptyHunyuan")
            || class_type.contains("WanVideoEmpty");

        if (width.is_none() || height.is_none()) && is_latent {
            if let Some(inputs) = node.get("inputs") {
                if width.is_none() {
                    width = inputs
                        .get("width")
                        .and_then(|v| v.as_u64())
                        .map(|w| w as u32);
                }
                if height.is_none() {
                    height = inputs
                        .get("height")
                        .and_then(|v| v.as_u64())
                        .map(|h| h as u32);
                }
            }
        }
    }

    // Fallback: search for prompt / text encode nodes (CLIPTextEncode, WanVideoTextEncode, HyVideoTextEncode, CogVideoTextEncode, etc.)
    if prompt.is_none() || negative_prompt.is_none() {
        for (_node_id, node) in nodes_map {
            let class_type = node
                .get("class_type")
                .and_then(|c| c.as_str())
                .unwrap_or_default();
            let is_text_node = class_type.contains("CLIPTextEncode")
                || class_type.contains("TextEncode")
                || class_type.contains("Prompt");
            if is_text_node {
                if let Some(inputs) = node.get("inputs") {
                    let text = inputs
                        .get("text")
                        .or_else(|| inputs.get("prompt"))
                        .or_else(|| inputs.get("astext"))
                        .or_else(|| inputs.get("positive_prompt"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_string());

                    if let Some(t) = text {
                        if !t.is_empty() {
                            let is_negative = class_type.to_lowercase().contains("negative")
                                || inputs.get("negative").is_some();
                            if is_negative && negative_prompt.is_none() {
                                negative_prompt = Some(t);
                            } else if prompt.is_none() {
                                prompt = Some(t);
                            } else if negative_prompt.is_none() && prompt.as_deref() != Some(&t) {
                                negative_prompt = Some(t);
                            }
                        }
                    }
                }
            }
        }
    }

    if prompt.is_none() && model_name.is_none() && steps.is_none() {
        return None;
    }

    Some(ExtractedMetadata {
        format: MetadataFormat::ComfyUI,
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

/// Recursively or directly extract text from a CLIPTextEncode node in the map.
fn extract_clip_text(nodes_map: &serde_json::Map<String, Value>, node_id: &str) -> Option<String> {
    let node = nodes_map.get(node_id)?;
    let inputs = node.get("inputs")?;

    // Direct text input
    if let Some(text) = inputs.get("text").and_then(|v| v.as_str()) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // SDXL dual clip text
    if let (Some(text_g), Some(text_l)) = (
        inputs.get("text_g").and_then(|v| v.as_str()),
        inputs.get("text_l").and_then(|v| v.as_str()),
    ) {
        let combined = format!("{text_g}, {text_l}").trim().to_string();
        if !combined.is_empty() {
            return Some(combined);
        }
    }

    // Conditioning link (e.g. ConditioningSetArea, ConditioningConcat)
    if let Some(cond_link) = inputs.get("conditioning").and_then(|v| v.as_array()) {
        if let Some(next_id) = cond_link.first().and_then(|v| v.as_str()) {
            return extract_clip_text(nodes_map, next_id);
        }
    }

    None
}

/// Parse ComfyUI workflow JSON (array format: `{ "nodes": [ ... ] }`).
fn parse_comfyui_workflow(root: &Value, json_str: &str) -> Option<ExtractedMetadata> {
    let nodes = root.get("nodes")?.as_array()?;

    let mut prompt = None;
    let mut negative_prompt = None;
    let mut steps = None;
    let mut cfg_scale = None;
    let mut seed = None;
    let mut sampler = None;
    let mut model_name = None;
    let mut width = None;
    let mut height = None;

    for node in nodes {
        let node_type = node
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if node_type.contains("KSampler") {
            if let Some(widgets) = node.get("widgets_values").and_then(|v| v.as_array()) {
                // Typical KSampler widgets: [seed, control_after_generate, steps, cfg, sampler_name, scheduler, denoise]
                if seed.is_none() && !widgets.is_empty() {
                    seed = widgets[0].as_u64().map(|s| s.to_string());
                }
                if steps.is_none() && widgets.len() > 2 {
                    steps = widgets[2].as_u64().map(|s| s as u32);
                }
                if cfg_scale.is_none() && widgets.len() > 3 {
                    cfg_scale = widgets[3].as_f64();
                }
                if sampler.is_none() && widgets.len() > 4 {
                    let s_name = widgets[4].as_str();
                    let sched = widgets.get(5).and_then(|v| v.as_str());
                    sampler = match (s_name, sched) {
                        (Some(s), Some(sch)) if !sch.is_empty() && sch != "normal" => {
                            Some(format!("{s}_{sch}"))
                        }
                        (Some(s), _) => Some(s.to_string()),
                        _ => None,
                    };
                }
            }
        }

        if model_name.is_none() && node_type.contains("CheckpointLoader") {
            if let Some(widgets) = node.get("widgets_values").and_then(|v| v.as_array()) {
                if let Some(ckpt) = widgets.first().and_then(|v| v.as_str()) {
                    model_name = Some(ckpt.to_string());
                }
            }
        }

        if (width.is_none() || height.is_none()) && node_type.contains("EmptyLatentImage") {
            if let Some(widgets) = node.get("widgets_values").and_then(|v| v.as_array()) {
                if width.is_none() && !widgets.is_empty() {
                    width = widgets[0].as_u64().map(|w| w as u32);
                }
                if height.is_none() && widgets.len() > 1 {
                    height = widgets[1].as_u64().map(|h| h as u32);
                }
            }
        }

        if node_type.contains("CLIPTextEncode") {
            if let Some(widgets) = node.get("widgets_values").and_then(|v| v.as_array()) {
                if let Some(text) = widgets.first().and_then(|v| v.as_str()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let title = node
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_lowercase();
                        if title.contains("negative") || title.contains("neg") {
                            negative_prompt = Some(trimmed.to_string());
                        } else if prompt.is_none() {
                            prompt = Some(trimmed.to_string());
                        } else if negative_prompt.is_none() {
                            negative_prompt = Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    if prompt.is_none() && model_name.is_none() && steps.is_none() {
        return None;
    }

    Some(ExtractedMetadata {
        format: MetadataFormat::ComfyUI,
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
    fn parses_comfyui_prompt_json() {
        let json = r#"{
            "3": {
                "class_type": "KSampler",
                "inputs": {
                    "seed": 8492049,
                    "steps": 28,
                    "cfg": 7.5,
                    "sampler_name": "dpmpp_2m",
                    "scheduler": "karras",
                    "positive": ["6", 0],
                    "negative": ["7", 0]
                }
            },
            "4": {
                "class_type": "CheckpointLoaderSimple",
                "inputs": {
                    "ckpt_name": "v1-5-pruned-emaonly.safetensors"
                }
            },
            "5": {
                "class_type": "EmptyLatentImage",
                "inputs": {
                    "width": 768,
                    "height": 512
                }
            },
            "6": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "cyberpunk city in rain, neon lights, masterpiece"
                }
            },
            "7": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "blurry, low quality, distortion"
                }
            }
        }"#;

        let meta = parse_comfyui(json).expect("parsed");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(
            meta.prompt.as_deref(),
            Some("cyberpunk city in rain, neon lights, masterpiece")
        );
        assert_eq!(
            meta.negative_prompt.as_deref(),
            Some("blurry, low quality, distortion")
        );
        assert_eq!(meta.steps, Some(28));
        assert_eq!(meta.cfg_scale, Some(7.5));
        assert_eq!(meta.sampler.as_deref(), Some("dpmpp_2m_karras"));
        assert_eq!(meta.seed.as_deref(), Some("8492049"));
        assert_eq!(
            meta.model_name.as_deref(),
            Some("v1-5-pruned-emaonly.safetensors")
        );
        assert_eq!(meta.width, Some(768));
        assert_eq!(meta.height, Some(512));
    }
}
