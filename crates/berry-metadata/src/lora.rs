//! Extraction of LoRA model references from generation prompts and ComfyUI workflow graphs.

use berry_domain::DetectedLora;
use serde_json::Value;

/// Extract all LoRAs detected from prompt text (`<lora:name:weight>`) and/or ComfyUI JSON graph.
pub fn extract_loras(prompt: Option<&str>, raw_json: Option<&str>) -> Vec<DetectedLora> {
    let mut results: Vec<DetectedLora> = Vec::new();

    // 1. Parse prompt text
    if let Some(text) = prompt {
        for lora in parse_prompt_loras(text) {
            if !results
                .iter()
                .any(|r| r.name.eq_ignore_ascii_case(&lora.name))
            {
                results.push(lora);
            }
        }
    }

    // 2. Parse raw ComfyUI JSON if available
    if let Some(json_str) = raw_json {
        for lora in parse_comfyui_loras(json_str) {
            if !results
                .iter()
                .any(|r| r.name.eq_ignore_ascii_case(&lora.name))
            {
                results.push(lora);
            }
        }
    }

    results
}

/// Normalize a raw LoRA path/filename into a canonical model name.
/// E.g. "xl\\styles\\anime_lineart.safetensors" -> "anime_lineart"
pub fn clean_lora_name(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('"').trim_matches('\'');
    // Take basename after slash or backslash
    let base = match trimmed.rfind(['/', '\\']) {
        Some(idx) => &trimmed[idx + 1..],
        None => trimmed,
    };
    // Strip common model extensions
    let lower = base.to_ascii_lowercase();
    for ext in &[".safetensors", ".pt", ".ckpt", ".bin"] {
        if lower.ends_with(ext) {
            return base[..base.len() - ext.len()].to_string();
        }
    }
    base.to_string()
}

/// Parse `<lora:NAME:WEIGHT>` tags from prompt text.
pub fn parse_prompt_loras(text: &str) -> Vec<DetectedLora> {
    let mut detected = Vec::new();
    let lower_text = text.to_ascii_lowercase();
    let tag = "<lora:";
    let mut search_start = 0;

    while let Some(idx) = lower_text[search_start..].find(tag) {
        let abs_start = search_start + idx + tag.len();
        if let Some(end_offset) = text[abs_start..].find('>') {
            let inner = &text[abs_start..abs_start + end_offset];
            let parts: Vec<&str> = inner.split(':').map(|s| s.trim()).collect();
            if let Some(raw_name) = parts.first() {
                let name = clean_lora_name(raw_name);
                if !name.is_empty() {
                    let weight = parts
                        .get(1)
                        .and_then(|w| w.parse::<f64>().ok())
                        .unwrap_or(1.0);

                    if !detected
                        .iter()
                        .any(|d: &DetectedLora| d.name.eq_ignore_ascii_case(&name))
                    {
                        detected.push(DetectedLora {
                            name,
                            weight,
                            model: None,
                        });
                    }
                }
            }
            search_start = abs_start + end_offset + 1;
        } else {
            break;
        }
    }

    detected
}

/// Parse ComfyUI workflow/prompt JSON to extract LoRAs from loader nodes.
pub fn parse_comfyui_loras(json_str: &str) -> Vec<DetectedLora> {
    let mut detected = Vec::new();
    let Ok(root) = serde_json::from_str::<Value>(json_str) else {
        return detected;
    };

    // Case 1: Prompt format - { "prompt": { ... } } or directly { "1": { ... } }
    if let Some(prompt_map) = root
        .get("prompt")
        .and_then(|p| p.as_object())
        .or_else(|| root.as_object())
    {
        for (_node_id, node) in prompt_map {
            let class_type = node
                .get("class_type")
                .and_then(|c| c.as_str())
                .unwrap_or_default();

            if class_type.contains("LoraLoader") || class_type.contains("LoRA") {
                if let Some(inputs) = node.get("inputs") {
                    let raw_name = inputs
                        .get("lora_name")
                        .or_else(|| inputs.get("lora"))
                        .and_then(|v| v.as_str());

                    if let Some(rn) = raw_name {
                        let name = clean_lora_name(rn);
                        if !name.is_empty() {
                            let weight = inputs
                                .get("strength_model")
                                .or_else(|| inputs.get("strength"))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(1.0);

                            if !detected
                                .iter()
                                .any(|d: &DetectedLora| d.name.eq_ignore_ascii_case(&name))
                            {
                                detected.push(DetectedLora {
                                    name,
                                    weight,
                                    model: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Case 2: Workflow format with "nodes" array
    if let Some(nodes_arr) = root.get("nodes").and_then(|n| n.as_array()) {
        for node in nodes_arr {
            let node_type = node
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or_default();

            if node_type.contains("LoraLoader") || node_type.contains("LoRA") {
                if let Some(widgets) = node.get("widgets_values").and_then(|w| w.as_array()) {
                    if let Some(raw_name) = widgets.first().and_then(|v| v.as_str()) {
                        let name = clean_lora_name(raw_name);
                        if !name.is_empty() {
                            let weight = widgets.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0);

                            if !detected
                                .iter()
                                .any(|d: &DetectedLora| d.name.eq_ignore_ascii_case(&name))
                            {
                                detected.push(DetectedLora {
                                    name,
                                    weight,
                                    model: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    detected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_lora_name() {
        assert_eq!(clean_lora_name("my_lora"), "my_lora");
        assert_eq!(clean_lora_name("subfolder/my_lora.safetensors"), "my_lora");
        assert_eq!(clean_lora_name("windows\\path\\model.ckpt"), "model");
        assert_eq!(clean_lora_name("\"quoted_name.pt\""), "quoted_name");
    }

    #[test]
    fn test_parse_prompt_loras() {
        let prompt = "1girl, solo, masterpiece, <lora:detail_tweaker:0.75>, <lora:anime_outline_v1:1.0:0.8>, <lora:flat_color>, photo";
        let loras = parse_prompt_loras(prompt);
        assert_eq!(loras.len(), 3);
        assert_eq!(loras[0].name, "detail_tweaker");
        assert_eq!(loras[0].weight, 0.75);
        assert_eq!(loras[1].name, "anime_outline_v1");
        assert_eq!(loras[1].weight, 1.0);
        assert_eq!(loras[2].name, "flat_color");
        assert_eq!(loras[2].weight, 1.0);
    }

    #[test]
    fn test_parse_comfyui_loras() {
        let json = r#"{
            "10": {
                "class_type": "LoraLoader",
                "inputs": {
                    "lora_name": "characters/miku_v2.safetensors",
                    "strength_model": 0.85
                }
            }
        }"#;
        let loras = parse_comfyui_loras(json);
        assert_eq!(loras.len(), 1);
        assert_eq!(loras[0].name, "miku_v2");
        assert_eq!(loras[0].weight, 0.85);
    }
}
