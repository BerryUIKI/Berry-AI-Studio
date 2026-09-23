use serde::{Deserialize, Serialize};

/// A similarity search match result associating an image file ID with its similarity score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimilarityMatch {
    /// Database row ID of the matched image file.
    pub file_id: i64,
    /// Cosine similarity score.
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similarity_match_serde_roundtrip() {
        let original = SimilarityMatch {
            file_id: 42,
            score: 0.85,
        };
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: SimilarityMatch = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
