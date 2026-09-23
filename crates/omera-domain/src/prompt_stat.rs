use serde::{Deserialize, Serialize};

/// Frequency and statistics for a prompt keyword.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptStat {
    /// The prompt token or phrase.
    pub text: String,
    /// Number of images using this token.
    pub count: usize,
}
