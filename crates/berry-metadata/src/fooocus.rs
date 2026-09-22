//! Fooocus / FooocusMRE metadata parser.

use berry_domain::{ExtractedMetadata, MetadataFormat};

/// Attempt to parse Fooocus formatted metadata parameters.
pub fn parse_fooocus(text: &str) -> Option<ExtractedMetadata> {
    if !text.contains("Prompt:")
        && !text.contains("Base Model:")
        && !text.contains("Fooocus")
        && !text.contains("Resolution:")
    {
        return None;
    }

    let mut prompt = None;
    let mut negative_prompt = None;
    let mut steps = None;
    let mut cfg_scale = None;
    let mut seed = None;
    let mut sampler = None;
    let mut model_name = None;
    let mut width = None;
    let mut height = None;

    let mut in_prompt = false;
    let mut in_neg = false;
    let mut prompt_lines = Vec::new();
    let mut neg_lines = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("Prompt:")
            .or_else(|| trimmed.strip_prefix("prompt:"))
        {
            in_prompt = true;
            in_neg = false;
            let val = rest.trim();
            if !val.is_empty() {
                prompt_lines.push(val);
            }
            continue;
        }

        if let Some(rest) = trimmed
            .strip_prefix("Negative:")
            .or_else(|| trimmed.strip_prefix("Negative Prompt:"))
            .or_else(|| trimmed.strip_prefix("negative_prompt:"))
        {
            in_prompt = false;
            in_neg = true;
            let val = rest.trim();
            if !val.is_empty() {
                neg_lines.push(val);
            }
            continue;
        }

        // Check if this line is a key: value parameter
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim();
            let val = trimmed[idx + 1..].trim();

            match key.to_lowercase().as_str() {
                "base model" | "model" => {
                    in_prompt = false;
                    in_neg = false;
                    model_name = Some(val.to_string());
                }
                "steps" => {
                    in_prompt = false;
                    in_neg = false;
                    steps = val.parse::<u32>().ok();
                }
                "cfg scale" | "cfg" | "guidance scale" => {
                    in_prompt = false;
                    in_neg = false;
                    cfg_scale = val.parse::<f64>().ok();
                }
                "seed" => {
                    in_prompt = false;
                    in_neg = false;
                    seed = Some(val.to_string());
                }
                "sampler" => {
                    in_prompt = false;
                    in_neg = false;
                    sampler = Some(val.to_string());
                }
                "scheduler" => {
                    in_prompt = false;
                    in_neg = false;
                    if let Some(ref mut s) = sampler {
                        if !val.is_empty() && val != "normal" {
                            *s = format!("{s}_{val}");
                        }
                    }
                }
                "resolution" => {
                    in_prompt = false;
                    in_neg = false;
                    // parses "(1024, 1024)" or "1024x1024" or "1024, 1024"
                    let cleaned = val.replace(['(', ')', ' '], "");
                    if let Some((w_str, h_str)) =
                        cleaned.split_once('x').or_else(|| cleaned.split_once(','))
                    {
                        width = w_str.parse::<u32>().ok();
                        height = h_str.parse::<u32>().ok();
                    }
                }
                _ => {
                    if in_prompt {
                        prompt_lines.push(trimmed);
                    } else if in_neg {
                        neg_lines.push(trimmed);
                    }
                }
            }
        } else if in_prompt {
            prompt_lines.push(trimmed);
        } else if in_neg {
            neg_lines.push(trimmed);
        }
    }

    if !prompt_lines.is_empty() {
        prompt = Some(prompt_lines.join(" "));
    }
    if !neg_lines.is_empty() {
        negative_prompt = Some(neg_lines.join(" "));
    }

    if prompt.is_none() && model_name.is_none() && steps.is_none() {
        return None;
    }

    Some(ExtractedMetadata {
        format: MetadataFormat::Fooocus,
        parameters: Some(text.to_string()),
        raw: Some(text.to_string()),
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
    fn parses_fooocus_text() {
        let text = "Prompt: ancient mystical forest, glowing trees\nNegative: blurry, distorted\nResolution: (1024, 768)\nBase Model: juggernautXL_v8.safetensors\nSampler: dpmpp_2m_sde_gpu\nScheduler: karras\nSeed: 981273912\nSteps: 30\nCFG scale: 4.0";
        let meta = parse_fooocus(text).expect("parsed");
        assert_eq!(meta.format, MetadataFormat::Fooocus);
        assert_eq!(
            meta.prompt.as_deref(),
            Some("ancient mystical forest, glowing trees")
        );
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry, distorted"));
        assert_eq!(meta.steps, Some(30));
        assert_eq!(meta.cfg_scale, Some(4.0));
        assert_eq!(meta.sampler.as_deref(), Some("dpmpp_2m_sde_gpu_karras"));
        assert_eq!(meta.seed.as_deref(), Some("981273912"));
        assert_eq!(
            meta.model_name.as_deref(),
            Some("juggernautXL_v8.safetensors")
        );
        assert_eq!(meta.width, Some(1024));
        assert_eq!(meta.height, Some(768));
    }
}
