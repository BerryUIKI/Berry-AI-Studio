//! Pure prompt-similarity planning for automatic image stacks.

use std::collections::HashSet;

/// Minimal file data needed to plan prompt-based stacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptStackCandidate {
    pub file_id: i64,
    pub folder_id: i64,
    pub modified_at: i64,
    pub prompt: String,
    pub model_name: Option<String>,
    pub is_stacked: bool,
}

/// A mutation-free grouping plan that storage adapters can apply transactionally.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptStackPlan {
    pub groups: Vec<Vec<i64>>,
    pub eligible_images: usize,
    pub skipped_without_prompt: usize,
}

fn normalize_prompt_fragment(fragment: &str) -> Option<String> {
    let mut normalized = fragment
        .trim()
        .trim_matches(|character: char| {
            character.is_whitespace() || matches!(character, '(' | ')' | '[' | ']' | '{' | '}')
        })
        .to_lowercase();

    if let Some((text, weight)) = normalized.rsplit_once(':') {
        if weight.trim().parse::<f32>().is_ok() {
            normalized = text.trim().to_string();
        }
    }

    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn tokenize_prompt(prompt: &str) -> HashSet<String> {
    prompt
        .split([',', ';', '\n', '\r', '\t', '|'])
        .filter_map(normalize_prompt_fragment)
        .collect()
}

fn prompt_jaccard_similarity(left: &HashSet<String>, right: &HashSet<String>) -> f32 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(right).count();
    let union = left.union(right).count();
    intersection as f32 / union as f32
}

fn normalized_model(model: Option<&str>) -> Option<String> {
    model
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_lowercase)
}

/// Plan stacks inside each folder using a time-bounded prompt similarity window.
///
/// Empty prompts and existing stack members are never grouped. Sorting by folder
/// and timestamp lets the inner scan stop as soon as it leaves the configured
/// time window instead of comparing every file with every other file.
pub fn plan_prompt_stacks(
    candidates: &[PromptStackCandidate],
    similarity_threshold: f32,
    time_window_secs: i64,
) -> PromptStackPlan {
    let threshold = if similarity_threshold.is_finite() {
        similarity_threshold.clamp(0.0, 1.0)
    } else {
        0.85
    };

    let mut skipped_without_prompt = 0;
    let mut prepared = candidates
        .iter()
        .filter(|candidate| !candidate.is_stacked)
        .filter_map(|candidate| {
            let tokens = tokenize_prompt(&candidate.prompt);
            if tokens.is_empty() {
                skipped_without_prompt += 1;
                return None;
            }
            Some((
                candidate,
                tokens,
                normalized_model(candidate.model_name.as_deref()),
            ))
        })
        .collect::<Vec<_>>();
    prepared.sort_by_key(|(candidate, _, _)| {
        (
            candidate.folder_id,
            candidate.modified_at,
            candidate.file_id,
        )
    });

    let eligible_images = prepared.len();
    let mut assigned = HashSet::new();
    let mut groups = Vec::new();

    for index in 0..prepared.len() {
        let (anchor, anchor_tokens, anchor_model) = &prepared[index];
        if assigned.contains(&anchor.file_id) {
            continue;
        }

        let mut group = vec![anchor.file_id];
        for (candidate, tokens, model) in prepared.iter().skip(index + 1) {
            if candidate.folder_id != anchor.folder_id {
                break;
            }
            let elapsed = candidate.modified_at.saturating_sub(anchor.modified_at);
            if time_window_secs > 0 && elapsed > time_window_secs {
                break;
            }
            if assigned.contains(&candidate.file_id) {
                continue;
            }
            if anchor_model.is_some() && model.is_some() && anchor_model != model {
                continue;
            }
            if prompt_jaccard_similarity(anchor_tokens, tokens) >= threshold {
                group.push(candidate.file_id);
                assigned.insert(candidate.file_id);
            }
        }

        if group.len() >= 2 {
            assigned.insert(anchor.file_id);
            groups.push(group);
        }
    }

    PromptStackPlan {
        groups,
        eligible_images,
        skipped_without_prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: i64, folder: i64, modified_at: i64, prompt: &str) -> PromptStackCandidate {
        PromptStackCandidate {
            file_id: id,
            folder_id: folder,
            modified_at,
            prompt: prompt.to_string(),
            model_name: Some("Model A".to_string()),
            is_stacked: false,
        }
    }

    #[test]
    fn ignores_empty_prompts_and_existing_stacks() {
        let mut stacked = candidate(3, 1, 12, "cat, portrait");
        stacked.is_stacked = true;
        let plan = plan_prompt_stacks(
            &[candidate(1, 1, 10, ""), candidate(2, 1, 11, "   "), stacked],
            0.8,
            60,
        );
        assert!(plan.groups.is_empty());
        assert_eq!(plan.eligible_images, 0);
        assert_eq!(plan.skipped_without_prompt, 2);
    }

    #[test]
    fn normalizes_prompt_weights_and_groups_only_inside_a_folder() {
        let plan = plan_prompt_stacks(
            &[
                candidate(1, 1, 10, "(masterpiece:1.2), blue sky, cat"),
                candidate(2, 1, 20, "masterpiece, blue   sky, cat"),
                candidate(3, 2, 20, "masterpiece, blue sky, cat"),
            ],
            1.0,
            60,
        );
        assert_eq!(plan.groups, vec![vec![1, 2]]);
        assert_eq!(plan.eligible_images, 3);
    }

    #[test]
    fn respects_time_windows_and_model_boundaries() {
        let mut other_model = candidate(3, 1, 20, "cat, portrait");
        other_model.model_name = Some("Model B".to_string());
        let plan = plan_prompt_stacks(
            &[
                candidate(1, 1, 10, "cat, portrait"),
                candidate(2, 1, 100, "cat, portrait"),
                other_model,
            ],
            1.0,
            30,
        );
        assert!(plan.groups.is_empty());
    }
}
