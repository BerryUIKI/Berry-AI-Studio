use image::DynamicImage;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;

use crate::preprocess::preprocess_image;
use crate::tags::{load_tags_csv, TagCategory, TagPrediction, TaggerConfig, TaggerTag, TagsError};

#[derive(Error, Debug)]
pub enum TaggerError {
    #[error("Failed to load model: {0}")]
    ModelLoad(String),
    #[error("Inference execution failed: {0}")]
    Inference(String),
    #[error("Tags error: {0}")]
    Tags(#[from] TagsError),
    #[error("Image load or decode error: {0}")]
    Image(#[from] image::ImageError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid model input/output shapes")]
    ShapeError,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_path: PathBuf,
    pub tags_path: PathBuf,
    pub input_size: u32,
    pub is_nchw: bool,
}

pub struct Wd14Tagger {
    session: Mutex<Session>,
    tags: Vec<TaggerTag>,
    pub model_info: ModelInfo,
}

impl Wd14Tagger {
    /// Load WD14 ONNX model and matching selected_tags.csv
    pub fn load(model_path: &Path, tags_path: &Path) -> Result<Self, TaggerError> {
        let tags = load_tags_csv(tags_path)?;

        let session = Session::builder()
            .map_err(|e| TaggerError::ModelLoad(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| TaggerError::ModelLoad(e.to_string()))?
            .with_intra_threads(4)
            .map_err(|e| TaggerError::ModelLoad(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| TaggerError::ModelLoad(e.to_string()))?;

        let input_size = 448u32;
        let is_nchw = false;

        let model_name = model_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("wd14-model")
            .to_string();

        let model_info = ModelInfo {
            name: model_name,
            model_path: model_path.to_path_buf(),
            tags_path: tags_path.to_path_buf(),
            input_size,
            is_nchw,
        };

        Ok(Self {
            session: Mutex::new(session),
            tags,
            model_info,
        })
    }

    /// Run inference on a dynamic image and extract tags above confidence thresholds
    pub fn predict(
        &self,
        img: &DynamicImage,
        config: &TaggerConfig,
    ) -> Result<Vec<TagPrediction>, TaggerError> {
        let (shape, data) =
            preprocess_image(img, self.model_info.input_size, self.model_info.is_nchw);

        // Build tensor
        let shape_dims: Vec<i64> = shape.into_iter().map(|v| v as i64).collect();
        let tensor = Tensor::from_array((shape_dims, data.into_boxed_slice()))
            .map_err(|e| TaggerError::Inference(e.to_string()))?;

        let mut session = self
            .session
            .lock()
            .map_err(|e| TaggerError::Inference(e.to_string()))?;

        let input_name = session
            .inputs()
            .first()
            .map(|inp| inp.name().to_string())
            .unwrap_or_else(|| "input_1:0".to_string());

        let outputs = session
            .run(ort::inputs![input_name => tensor])
            .map_err(|e| TaggerError::Inference(e.to_string()))?;

        let output_tensor = &outputs[0];

        let (_shape, raw_scores) = output_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| TaggerError::Inference(e.to_string()))?;

        let mut predictions = Vec::new();

        for (idx, tag) in self.tags.iter().enumerate() {
            if idx >= raw_scores.len() {
                break;
            }

            let raw_val = raw_scores[idx];
            // If the model produces unnormalized logits, apply sigmoid;
            // If already in 0..1, keep as-is.
            let confidence = if !(0.0..=1.0).contains(&raw_val) {
                1.0 / (1.0 + (-raw_val).exp())
            } else {
                raw_val
            };

            // Category-based thresholding
            let threshold = match tag.category {
                TagCategory::General => config.general_threshold,
                TagCategory::Character => config.character_threshold,
                TagCategory::Rating => {
                    if !config.include_rating {
                        continue;
                    }
                    config.general_threshold
                }
                TagCategory::Unknown(_) => config.general_threshold,
            };

            if confidence >= threshold {
                predictions.push(TagPrediction {
                    name: tag.name.clone(),
                    category: tag.category,
                    confidence,
                });
            }
        }

        // Sort descending by confidence
        predictions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if predictions.len() > config.max_tags {
            predictions.truncate(config.max_tags);
        }

        Ok(predictions)
    }

    /// Run inference on an image file by path
    pub fn predict_file(
        &self,
        path: &Path,
        config: &TaggerConfig,
    ) -> Result<Vec<TagPrediction>, TaggerError> {
        let img = image::open(path)?;
        self.predict(&img, config)
    }

    pub fn tags_count(&self) -> usize {
        self.tags.len()
    }
}
