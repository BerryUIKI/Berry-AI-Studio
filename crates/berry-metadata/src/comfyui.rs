//! ComfyUI metadata parsing from node graph JSON (`prompt` or `workflow` chunks).

use std::collections::HashSet;

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

    // Fallback: search for prompt / text encode nodes (CLIPTextEncode, CLIPTextEncodeFlux, WanVideoTextEncode, HyVideoTextEncode, CogVideoTextEncode, etc.)
    if prompt.is_none() || negative_prompt.is_none() {
        for (node_id, node) in nodes_map {
            let class_type = node
                .get("class_type")
                .and_then(|c| c.as_str())
                .unwrap_or_default();
            let is_text_node = class_type.contains("CLIPTextEncode")
                || class_type.contains("TextEncode")
                || class_type.contains("Prompt");
            if is_text_node {
                if let Some(t) = extract_clip_text(nodes_map, node_id) {
                    if !t.is_empty() {
                        let is_negative = class_type.to_lowercase().contains("negative")
                            || node.get("inputs").and_then(|i| i.get("negative")).is_some();
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

/// Recursively or directly extract text from a prompt/conditioning/text node in the map.
fn extract_clip_text(nodes_map: &serde_json::Map<String, Value>, node_id: &str) -> Option<String> {
    let mut visited = HashSet::new();
    resolve_node_text(nodes_map, node_id, &mut visited, 0)
}

fn resolve_value_or_link(
    nodes_map: &serde_json::Map<String, Value>,
    val: &Value,
    visited: &mut HashSet<String>,
    depth: usize,
) -> Option<String> {
    if depth > 10 {
        return None;
    }
    if let Some(text) = val.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    } else if let Some(arr) = val.as_array() {
        if let Some(target_id) = arr.first().and_then(|v| v.as_str()) {
            return resolve_node_text(nodes_map, target_id, visited, depth + 1);
        }
    }
    None
}

fn resolve_node_text(
    nodes_map: &serde_json::Map<String, Value>,
    node_id: &str,
    visited: &mut HashSet<String>,
    depth: usize,
) -> Option<String> {
    if depth > 10 || !visited.insert(node_id.to_string()) {
        return None;
    }

    let node = nodes_map.get(node_id)?;
    let inputs = node.get("inputs")?;
    let class_type = node
        .get("class_type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    // 1. Direct text input or linked text input (e.g. CLIPTextEncode, WanVideoTextEncode)
    if let Some(text_val) = inputs
        .get("text")
        .or_else(|| inputs.get("prompt"))
        .or_else(|| inputs.get("astext"))
    {
        if let Some(text) = resolve_value_or_link(nodes_map, text_val, visited, depth) {
            return Some(text);
        }
    }

    // 2. Flux CLIPTextEncodeFlux (clip_l, t5xxl, guidance, clip)
    if class_type.contains("Flux")
        || inputs.get("clip_l").is_some()
        || inputs.get("t5xxl").is_some()
    {
        let clip_l = inputs
            .get("clip_l")
            .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));
        let t5xxl = inputs
            .get("t5xxl")
            .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));
        match (clip_l, t5xxl) {
            (Some(l), Some(t5)) => {
                if l == t5 || t5.is_empty() {
                    return Some(l);
                }
                if l.is_empty() {
                    return Some(t5);
                }
                return Some(format!("{l}, {t5}"));
            }
            (Some(l), None) => return Some(l),
            (None, Some(t5)) => return Some(t5),
            (None, None) => {}
        }
    }

    // 3. SDXL dual clip text (text_g, text_l)
    let text_g = inputs
        .get("text_g")
        .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));
    let text_l = inputs
        .get("text_l")
        .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));
    match (text_g, text_l) {
        (Some(g), Some(l)) => {
            if g == l || l.is_empty() {
                return Some(g);
            }
            if g.is_empty() {
                return Some(l);
            }
            return Some(format!("{g}, {l}"));
        }
        (Some(g), None) => return Some(g),
        (None, Some(l)) => return Some(l),
        (None, None) => {}
    }

    // 4. ComfyUI-Easy-Use concatenation & string manipulation nodes (easy promptConcat, PromptConcat, etc.)
    if class_type.contains("promptConcat")
        || class_type.contains("PromptConcat")
        || class_type.contains("StringConcatenate")
        || class_type.contains("Text Concatenate")
        || class_type.contains("Concat")
    {
        let p1 = inputs
            .get("prompt1")
            .or_else(|| inputs.get("text1"))
            .or_else(|| inputs.get("string1"))
            .or_else(|| inputs.get("str1"))
            .or_else(|| inputs.get("a"))
            .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));

        let p2 = inputs
            .get("prompt2")
            .or_else(|| inputs.get("text2"))
            .or_else(|| inputs.get("string2"))
            .or_else(|| inputs.get("str2"))
            .or_else(|| inputs.get("b"))
            .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth));

        let sep = inputs
            .get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match (p1, p2) {
            (Some(s1), Some(s2)) => {
                if s1.is_empty() {
                    return Some(s2);
                }
                if s2.is_empty() {
                    return Some(s1);
                }
                return Some(format!("{s1}{sep}{s2}"));
            }
            (Some(s1), None) => return Some(s1),
            (None, Some(s2)) => return Some(s2),
            (None, None) => {}
        }
    }

    // 5. easy positive / easy negative
    if let Some(pos) = inputs
        .get("positive")
        .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth))
    {
        return Some(pos);
    }
    if let Some(neg) = inputs
        .get("negative")
        .and_then(|v| resolve_value_or_link(nodes_map, v, visited, depth))
    {
        return Some(neg);
    }

    // 6. PrimitiveNode / StringLiteral / String / Text values
    if let Some(val) = inputs
        .get("value")
        .or_else(|| inputs.get("string"))
        .or_else(|| inputs.get("positive_prompt"))
    {
        if let Some(text) = resolve_value_or_link(nodes_map, val, visited, depth) {
            return Some(text);
        }
    }

    // 7. Conditioning links (ConditioningSetArea, ConditioningConcat, ConditioningAverage, ConditioningCombine)
    for cond_key in [
        "conditioning",
        "conditioning_to",
        "conditioning_from",
        "conditioning_1",
        "conditioning_2",
    ] {
        if let Some(cond_link) = inputs.get(cond_key).and_then(|v| v.as_array()) {
            if let Some(next_id) = cond_link.first().and_then(|v| v.as_str()) {
                if let Some(text) = resolve_node_text(nodes_map, next_id, visited, depth + 1) {
                    return Some(text);
                }
            }
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

    #[test]
    fn parses_comfyui_flux_and_easyuse_nodes() {
        let json = r#"{
            "3": {
                "class_type": "KSampler",
                "inputs": {
                    "seed": 1234567,
                    "steps": 20,
                    "cfg": 3.5,
                    "sampler_name": "euler",
                    "scheduler": "simple",
                    "positive": ["5", 0],
                    "negative": ["8", 0]
                }
            },
            "4": {
                "class_type": "UNETLoader",
                "inputs": {
                    "unet_name": "flux1-dev.sft"
                }
            },
            "5": {
                "class_type": "CLIPTextEncodeFlux",
                "inputs": {
                    "clip_l": ["68", 0],
                    "t5xxl": ["68", 0],
                    "guidance": 3.5
                }
            },
            "8": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "ugly, watermark"
                }
            },
            "68": {
                "class_type": "easy promptConcat",
                "inputs": {
                    "prompt1": ["70", 0],
                    "prompt2": ["69", 0],
                    "separator": ", "
                }
            },
            "69": {
                "class_type": "easy positive",
                "inputs": {
                    "positive": "masterpiece, best quality"
                }
            },
            "70": {
                "class_type": "easy positive",
                "inputs": {
                    "positive": "1girl, solo, cherry blossoms"
                }
            }
        }"#;

        let meta = parse_comfyui(json).expect("parsed flux and easy-use workflow");
        assert_eq!(
            meta.prompt.as_deref(),
            Some("1girl, solo, cherry blossoms, masterpiece, best quality")
        );
        assert_eq!(meta.negative_prompt.as_deref(), Some("ugly, watermark"));
        assert_eq!(meta.model_name.as_deref(), Some("flux1-dev.sft"));
        assert_eq!(meta.steps, Some(20));
    }

    #[test]
    fn handles_comfyui_graph_cycle_without_overflow() {
        let json = r#"{
            "3": {
                "class_type": "KSampler",
                "inputs": {
                    "positive": ["10", 0]
                }
            },
            "10": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": ["11", 0]
                }
            },
            "11": {
                "class_type": "easy promptConcat",
                "inputs": {
                    "prompt1": ["10", 0],
                    "prompt2": "safe fallback text"
                }
            }
        }"#;

        let _meta = parse_comfyui(json);
    }
}
