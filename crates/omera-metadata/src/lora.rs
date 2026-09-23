//! Extraction of LoRA model references from generation prompts, WebUI parameter footers, and ComfyUI workflow graphs.

use std::collections::{BTreeMap, HashMap};

use omera_domain::DetectedLora;
use serde_json::Value;

/// Extract all LoRAs detected from prompt text (`<lora:name:weight>`) and/or ComfyUI JSON graph / parameter footer.
pub fn extract_loras(prompt: Option<&str>, raw_or_parameters: Option<&str>) -> Vec<DetectedLora> {
    extract_loras_full(prompt, raw_or_parameters, raw_or_parameters)
}

/// Full extraction checking prompt text, raw workflow JSON, and raw parameter strings.
pub fn extract_loras_full(
    prompt: Option<&str>,
    raw_json: Option<&str>,
    parameters: Option<&str>,
) -> Vec<DetectedLora> {
    let mut results: Vec<DetectedLora> = Vec::new();

    // 1. Parse prompt text
    if let Some(text) = prompt {
        for lora in parse_prompt_loras(text) {
            merge_detected_lora(&mut results, lora);
        }
        for lora in parse_parameter_footer_loras(text) {
            merge_detected_lora(&mut results, lora);
        }
    }

    // 2. Parse raw ComfyUI JSON if available
    if let Some(json_str) = raw_json {
        for lora in parse_comfyui_loras(json_str) {
            merge_detected_lora(&mut results, lora);
        }
    }

    // 3. Parse parameters string if available
    if let Some(param_text) = parameters {
        if raw_json != Some(param_text) {
            for lora in parse_comfyui_loras(param_text) {
                merge_detected_lora(&mut results, lora);
            }
        }
        for lora in parse_prompt_loras(param_text) {
            merge_detected_lora(&mut results, lora);
        }
        for lora in parse_parameter_footer_loras(param_text) {
            merge_detected_lora(&mut results, lora);
        }
    }

    results
}

/// Merge an incoming DetectedLora into an existing list without duplicates.
/// Populates missing hashes or non-default weights where applicable.
pub fn merge_detected_lora(results: &mut Vec<DetectedLora>, incoming: DetectedLora) {
    if let Some(existing) = results
        .iter_mut()
        .find(|r| r.name.eq_ignore_ascii_case(&incoming.name))
    {
        if existing.hash.is_none() && incoming.hash.is_some() {
            existing.hash = incoming.hash;
        }
        if (existing.weight - 1.0).abs() < f64::EPSILON
            && (incoming.weight - 1.0).abs() >= f64::EPSILON
        {
            existing.weight = incoming.weight;
        }
    } else {
        results.push(incoming);
    }
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
                            hash: None,
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
                                    hash: None,
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
                                    hash: None,
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

#[derive(Debug, PartialEq, Eq)]
enum IndexedKeyKind {
    Name,
    Weight,
    Hash,
}

fn classify_indexed_key(key_lower: &str) -> Option<(usize, IndexedKeyKind)> {
    let parts: Vec<&str> = key_lower
        .split([' ', '_', '-'])
        .filter(|s| !s.is_empty())
        .collect();

    let mut parsed_idx: Option<usize> = None;
    for part in &parts {
        if let Ok(num) = part.parse::<usize>() {
            parsed_idx = Some(num);
            break;
        } else if let Some(digits) = part.strip_prefix("lora") {
            if let Ok(num) = digits.parse::<usize>() {
                parsed_idx = Some(num);
                break;
            }
        }
    }

    let idx = parsed_idx?;

    if parts.contains(&"hash") {
        Some((idx, IndexedKeyKind::Hash))
    } else if parts.contains(&"weight") {
        Some((idx, IndexedKeyKind::Weight))
    } else if parts.contains(&"lora")
        || parts.iter().any(|p| p.starts_with("lora"))
        || parts.contains(&"model")
    {
        Some((idx, IndexedKeyKind::Name))
    } else {
        None
    }
}

fn is_plausible_hash(s: &str) -> bool {
    let trimmed = s.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 128
        && trimmed.chars().all(|c| c.is_ascii_alphanumeric())
}

fn split_settings_segments(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    let bytes = text.as_bytes();
    let len = text.len();

    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if b == b'"' {
            in_quotes = !in_quotes;
        } else if (b == b',' || b == b'\n') && !in_quotes {
            segments.push(&text[start..i]);
            start = i + 1;
        } else if in_quotes && b == b',' {
            // Guard against unclosed quote swallowing subsequent settings keys:
            let rest = &text[i + 1..];
            let trimmed_rest = rest.trim_start();
            if trimmed_rest.starts_with("Lora ")
                || trimmed_rest.starts_with("Steps:")
                || trimmed_rest.starts_with("Sampler:")
                || trimmed_rest.starts_with("Size:")
                || trimmed_rest.starts_with("Model:")
                || trimmed_rest.starts_with("Seed:")
            {
                in_quotes = false;
                segments.push(&text[start..i]);
                start = i + 1;
            }
        }
        i += 1;
    }
    if start < len {
        segments.push(&text[start..]);
    }
    segments
}

fn parse_lora_hashes_value(val: &str, hashes: &mut HashMap<String, String>) {
    let unquoted = val.trim().trim_matches('"').trim_matches('\'').trim();
    if unquoted.starts_with('{') && unquoted.ends_with('}') {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(unquoted) {
            for (k, v) in map {
                let name = clean_lora_name(k.strip_prefix("lora:").unwrap_or(&k));
                if let Some(h) = v.as_str() {
                    let clean_h = h.trim();
                    if is_plausible_hash(clean_h) {
                        hashes.insert(name.to_ascii_lowercase(), clean_h.to_string());
                    }
                }
            }
            return;
        }
    }

    for pair in unquoted.split(',') {
        let pair_trimmed = pair.trim();
        if let Some((raw_name, raw_hash)) = pair_trimmed.split_once(':') {
            let name = clean_lora_name(raw_name);
            let hash = raw_hash.trim().trim_matches('"').trim_matches('\'').trim();
            if !name.is_empty() && is_plausible_hash(hash) {
                hashes.insert(name.to_ascii_lowercase(), hash.to_string());
            }
        }
    }
}

/// Parse LoRA references and hashes from WebUI / Liblib parameter footers.
pub fn parse_parameter_footer_loras(text: &str) -> Vec<DetectedLora> {
    let mut detected = Vec::new();
    let mut lora_hashes: HashMap<String, String> = HashMap::new();
    let mut indexed_loras: BTreeMap<usize, (String, Option<f64>, Option<String>)> = BTreeMap::new();

    let segments = split_settings_segments(text);
    for seg in segments {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((raw_key, raw_val)) = trimmed.split_once(':') else {
            continue;
        };
        let key = raw_key.trim();
        let val = raw_val.trim();
        let key_lower = key.to_ascii_lowercase();

        // Check for "Lora hashes" / "Hashes"
        if key_lower == "lora hashes"
            || key_lower.ends_with(" lora hashes")
            || key_lower == "hashes"
        {
            parse_lora_hashes_value(val, &mut lora_hashes);
            continue;
        }

        // Check for indexed keys
        if let Some((idx, kind)) = classify_indexed_key(&key_lower) {
            match kind {
                IndexedKeyKind::Name => {
                    let entry = indexed_loras
                        .entry(idx)
                        .or_insert_with(|| (String::new(), None, None));
                    let clean_val = val.trim_matches('"').trim_matches('\'');
                    if let Some((n, w_str)) = clean_val.split_once(':') {
                        entry.0 = clean_lora_name(n);
                        if let Ok(w) = w_str.trim().parse::<f64>() {
                            entry.1 = Some(w);
                        }
                    } else {
                        entry.0 = clean_lora_name(clean_val);
                    }
                }
                IndexedKeyKind::Weight => {
                    if let Ok(w) = val.trim_matches('"').trim_matches('\'').parse::<f64>() {
                        let entry = indexed_loras
                            .entry(idx)
                            .or_insert_with(|| (String::new(), None, None));
                        entry.1 = Some(w);
                    }
                }
                IndexedKeyKind::Hash => {
                    let h = val.trim_matches('"').trim_matches('\'').trim().to_string();
                    if !h.is_empty() {
                        let entry = indexed_loras
                            .entry(idx)
                            .or_insert_with(|| (String::new(), None, None));
                        entry.2 = Some(h);
                    }
                }
            }
        }
    }

    // Convert indexed LoRAs to DetectedLora
    for (_idx, (raw_name, weight_opt, hash_opt)) in indexed_loras {
        let name = clean_lora_name(&raw_name);
        if name.is_empty() {
            continue;
        }
        let weight = weight_opt.unwrap_or(1.0);
        let hash = hash_opt.or_else(|| lora_hashes.remove(&name.to_ascii_lowercase()));
        if !detected
            .iter()
            .any(|d: &DetectedLora| d.name.eq_ignore_ascii_case(&name))
        {
            detected.push(DetectedLora {
                name,
                weight,
                hash,
                model: None,
            });
        }
    }

    // Any remaining lora_hashes not tied to an indexed LoRA
    for (name_lower, hash) in lora_hashes {
        if !detected
            .iter()
            .any(|d: &DetectedLora| d.name.eq_ignore_ascii_case(&name_lower))
        {
            detected.push(DetectedLora {
                name: name_lower,
                weight: 1.0,
                hash: Some(hash),
                model: None,
            });
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
        assert_eq!(loras[0].hash, None);
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
        assert_eq!(loras[0].hash, None);
    }

    #[test]
    fn test_parse_parameter_footer_lora_hashes() {
        let footer = r#"Steps: 28, Sampler: DPM++ 2M Karras, CFG scale: 7, Seed: 1716021952, Size: 832x1216, Model hash: abc123, Model: realisticVision, Lora hashes: "detail_tweaker: 9b422a5a0459, anime_lineart: 4d2843efc612", Version: v1.6.0"#;
        let loras = parse_parameter_footer_loras(footer);
        assert_eq!(loras.len(), 2);
        let dt = loras.iter().find(|l| l.name == "detail_tweaker").unwrap();
        assert_eq!(dt.hash.as_deref(), Some("9b422a5a0459"));
        assert_eq!(dt.weight, 1.0);

        let al = loras.iter().find(|l| l.name == "anime_lineart").unwrap();
        assert_eq!(al.hash.as_deref(), Some("4d2843efc612"));
    }

    #[test]
    fn test_parse_parameter_footer_indexed_liblib() {
        let footer = "masterpiece, 1girl, solo\nNegative prompt: easynegative\nSteps: 20, Sampler: Euler a, CFG scale: 7, Seed: 1234567, Size: 512x768, Model: counterfeitV30, Lora 1: JapaneseDollLikeness_v15, Lora Hash 1: 5882650085a5, Lora Weight 1: 0.65, Lora 2: koreanDollLikeness_v20, Lora Hash 2: 739b6008ab14, Lora Weight 2: 0.35";
        let loras = parse_parameter_footer_loras(footer);
        assert_eq!(loras.len(), 2);
        assert_eq!(loras[0].name, "JapaneseDollLikeness_v15");
        assert_eq!(loras[0].weight, 0.65);
        assert_eq!(loras[0].hash.as_deref(), Some("5882650085a5"));
        assert_eq!(loras[1].name, "koreanDollLikeness_v20");
        assert_eq!(loras[1].weight, 0.35);
        assert_eq!(loras[1].hash.as_deref(), Some("739b6008ab14"));
    }

    #[test]
    fn test_parse_parameter_footer_combined_name_weight() {
        let footer = r#"Steps: 25, Sampler: DPM++ SDE Karras, Lora 1: "style\gothic_art.safetensors:0.85", Lora Hash 1: e8a25c19d4"#;
        let loras = parse_parameter_footer_loras(footer);
        assert_eq!(loras.len(), 1);
        assert_eq!(loras[0].name, "gothic_art");
        assert_eq!(loras[0].weight, 0.85);
        assert_eq!(loras[0].hash.as_deref(), Some("e8a25c19d4"));
    }

    #[test]
    fn test_extract_loras_merges_prompt_and_footer() {
        let prompt = "1girl, solo, <lora:detail_tweaker:0.75>, <lora:another_lora:0.9>";
        let footer = r#"Steps: 28, Sampler: Euler, Lora hashes: "detail_tweaker: 9b422a5a0459", Lora 1: third_lora, Lora Hash 1: 12345678, Lora Weight 1: 0.5"#;

        let merged = extract_loras_full(Some(prompt), None, Some(footer));
        assert_eq!(merged.len(), 3);

        let dt = merged.iter().find(|l| l.name == "detail_tweaker").unwrap();
        assert_eq!(dt.weight, 0.75); // preserved prompt weight
        assert_eq!(dt.hash.as_deref(), Some("9b422a5a0459")); // populated from footer

        let an = merged.iter().find(|l| l.name == "another_lora").unwrap();
        assert_eq!(an.weight, 0.9);
        assert_eq!(an.hash, None);

        let th = merged.iter().find(|l| l.name == "third_lora").unwrap();
        assert_eq!(th.weight, 0.5);
        assert_eq!(th.hash.as_deref(), Some("12345678"));
    }

    #[test]
    fn test_parse_parameter_footer_malformed_resilience() {
        // Missing values, unclosed quotes, non-numeric weights, empty input
        let malformed = r#"Steps: 20, Lora hashes: "broken_hash: , trailing:, Lora 1: valid_name, Lora Weight 1: not_a_number, Lora Hash 1: , Lora 2:, Lora Weight 2: 0.8"#;
        let loras = parse_parameter_footer_loras(malformed);
        assert_eq!(loras.len(), 1);
        assert_eq!(loras[0].name, "valid_name");
        assert_eq!(loras[0].weight, 1.0); // defaulted from non-numeric

        // Empty string
        assert!(parse_parameter_footer_loras("").is_empty());
        assert!(parse_parameter_footer_loras("   \n\t ").is_empty());
        assert!(parse_parameter_footer_loras("Steps: 20, Sampler: Euler").is_empty());
    }
}
