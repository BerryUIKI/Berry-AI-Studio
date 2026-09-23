use image::DynamicImage;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

use crate::preprocess::{l2_normalize, preprocess_image};
use crate::tokenizer::{ClipTokenizer, TokenizerError};

#[derive(Error, Debug)]
pub enum ClipError {
    #[error("Failed to load model from {0}: {1}")]
    ModelLoad(String, String),
    #[error("Tokenizer error: {0}")]
    Tokenizer(#[from] TokenizerError),
    #[error("Inference execution failed: {0}")]
    Inference(String),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid model input or output shape")]
    InvalidShape,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipModelInfo {
    pub model_id: String,
    pub name: String,
    pub folder_path: PathBuf,
    pub visual_model_path: PathBuf,
    pub textual_model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub image_size: u32,
    pub embedding_dim: usize,
}

pub struct ClipEngine {
    visual_session: Mutex<Session>,
    textual_session: Mutex<Session>,
    tokenizer: ClipTokenizer,
    pub info: ClipModelInfo,
}

impl ClipEngine {
    /// Loads CLIP / SigLIP visual and textual models along with tokenizer from directory
    pub fn load_from_dir(dir: &Path) -> Result<Self, ClipError> {
        // Look for visual model
        let visual_candidates = ["visual.onnx", "vision_model.onnx", "model.onnx"];
        let mut visual_path = None;
        for c in &visual_candidates {
            let p = dir.join(c);
            if p.is_file() {
                visual_path = Some(p);
                break;
            }
        }
        let visual_path = visual_path.ok_or_else(|| {
            ClipError::ModelLoad(
                dir.to_string_lossy().to_string(),
                "No visual ONNX model found (visual.onnx, vision_model.onnx)".to_string(),
            )
        })?;

        // Look for textual model
        let textual_candidates = ["textual.onnx", "text_model.onnx"];
        let mut textual_path = None;
        for c in &textual_candidates {
            let p = dir.join(c);
            if p.is_file() {
                textual_path = Some(p);
                break;
            }
        }
        let textual_path = textual_path.ok_or_else(|| {
            ClipError::ModelLoad(
                dir.to_string_lossy().to_string(),
                "No textual ONNX model found (textual.onnx, text_model.onnx)".to_string(),
            )
        })?;

        // Look for tokenizer
        let tokenizer_candidates = ["tokenizer.json"];
        let mut tokenizer_path = None;
        for c in &tokenizer_candidates {
            let p = dir.join(c);
            if p.is_file() {
                tokenizer_path = Some(p);
                break;
            }
        }
        let tokenizer_path = tokenizer_path.ok_or_else(|| {
            ClipError::ModelLoad(
                dir.to_string_lossy().to_string(),
                "No tokenizer.json found".to_string(),
            )
        })?;

        let tokenizer = ClipTokenizer::from_file(&tokenizer_path)?;

        let visual_session = Session::builder()
            .map_err(|e| {
                ClipError::ModelLoad(visual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| {
                ClipError::ModelLoad(visual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .with_intra_threads(4)
            .map_err(|e| {
                ClipError::ModelLoad(visual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .commit_from_file(&visual_path)
            .map_err(|e| {
                ClipError::ModelLoad(visual_path.to_string_lossy().to_string(), e.to_string())
            })?;

        let textual_session = Session::builder()
            .map_err(|e| {
                ClipError::ModelLoad(textual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| {
                ClipError::ModelLoad(textual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .with_intra_threads(4)
            .map_err(|e| {
                ClipError::ModelLoad(textual_path.to_string_lossy().to_string(), e.to_string())
            })?
            .commit_from_file(&textual_path)
            .map_err(|e| {
                ClipError::ModelLoad(textual_path.to_string_lossy().to_string(), e.to_string())
            })?;

        let model_id = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("clip-vit-base-patch32")
            .to_string();

        let info = ClipModelInfo {
            model_id: model_id.clone(),
            name: model_id,
            folder_path: dir.to_path_buf(),
            visual_model_path: visual_path,
            textual_model_path: textual_path,
            tokenizer_path,
            image_size: 224,
            embedding_dim: 512,
        };

        Ok(Self {
            visual_session: Mutex::new(visual_session),
            textual_session: Mutex::new(textual_session),
            tokenizer,
            info,
        })
    }

    /// Encode image to normalized embedding vector
    pub fn encode_image(&self, img: &DynamicImage) -> Result<Vec<f32>, ClipError> {
        let (shape, data) = preprocess_image(img, self.info.image_size);
        let shape_dims: Vec<i64> = shape.into_iter().map(|v| v as i64).collect();
        let tensor = Tensor::from_array((shape_dims, data.into_boxed_slice()))
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        let mut session = self
            .visual_session
            .lock()
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        let input_name = session
            .inputs()
            .first()
            .map(|inp| inp.name().to_string())
            .unwrap_or_else(|| "pixel_values".to_string());

        let outputs = session
            .run(ort::inputs![input_name => tensor])
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        let output_tensor = &outputs[0];
        let (_shape, raw_embeddings) = output_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        Ok(l2_normalize(raw_embeddings))
    }

    /// Encode text query to normalized embedding vector
    pub fn encode_text(&self, text: &str) -> Result<Vec<f32>, ClipError> {
        let (input_ids, attention_mask) = self.tokenizer.encode_text(text)?;

        let ids_tensor = Tensor::from_array((
            [1i64, self.tokenizer.max_length as i64],
            input_ids.into_boxed_slice(),
        ))
        .map_err(|e| ClipError::Inference(e.to_string()))?;

        let mask_tensor = Tensor::from_array((
            [1i64, self.tokenizer.max_length as i64],
            attention_mask.into_boxed_slice(),
        ))
        .map_err(|e| ClipError::Inference(e.to_string()))?;

        let mut session = self
            .textual_session
            .lock()
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        let input_names: Vec<String> = session
            .inputs()
            .iter()
            .map(|inp| inp.name().to_string())
            .collect();

        let outputs = if input_names.len() >= 2 {
            // Usually [input_ids, attention_mask]
            let id_name = input_names
                .iter()
                .find(|n| n.contains("ids"))
                .cloned()
                .unwrap_or_else(|| input_names[0].clone());
            let mask_name = input_names
                .iter()
                .find(|n| n.contains("mask"))
                .cloned()
                .unwrap_or_else(|| input_names[1].clone());

            session
                .run(ort::inputs![id_name => ids_tensor, mask_name => mask_tensor])
                .map_err(|e| ClipError::Inference(e.to_string()))?
        } else {
            let id_name = input_names
                .first()
                .cloned()
                .unwrap_or_else(|| "input_ids".to_string());
            session
                .run(ort::inputs![id_name => ids_tensor])
                .map_err(|e| ClipError::Inference(e.to_string()))?
        };

        let output_tensor = &outputs[0];
        let (_shape, raw_embeddings) = output_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| ClipError::Inference(e.to_string()))?;

        Ok(l2_normalize(raw_embeddings))
    }
}
