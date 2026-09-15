use std::path::Path;
use thiserror::Error;
use tokenizers::Tokenizer;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("Failed to load tokenizer from {0}: {1}")]
    Load(String, String),
    #[error("Tokenization failed: {0}")]
    Encode(String),
}

pub struct ClipTokenizer {
    tokenizer: Tokenizer,
    pub max_length: usize,
}

impl ClipTokenizer {
    pub fn from_file(path: &Path) -> Result<Self, TokenizerError> {
        let tokenizer = Tokenizer::from_file(path)
            .map_err(|e| TokenizerError::Load(path.to_string_lossy().to_string(), e.to_string()))?;

        Ok(Self {
            tokenizer,
            max_length: 77,
        })
    }

    /// Tokenizes input text into standard CLIP shape: [1, max_length] of i64 or i32.
    /// Handles truncation to max_length and padding.
    pub fn encode_text(&self, text: &str) -> Result<(Vec<i64>, Vec<i64>), TokenizerError> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| TokenizerError::Encode(e.to_string()))?;

        let raw_ids = encoding.get_ids();
        let raw_mask = encoding.get_attention_mask();

        let mut input_ids = vec![0i64; self.max_length];
        let mut attention_mask = vec![0i64; self.max_length];

        let copy_len = raw_ids.len().min(self.max_length);
        for i in 0..copy_len {
            input_ids[i] = raw_ids[i] as i64;
            attention_mask[i] = raw_mask[i] as i64;
        }

        Ok((input_ids, attention_mask))
    }
}
