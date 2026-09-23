//! Domain types for Image Stacking and burst grouping.

use serde::{Deserialize, Serialize};

/// Summary information about an image stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackSummary {
    /// Unique UUID string identifying this stack group.
    pub stack_id: String,
    /// Total count of images in this stack.
    pub count: usize,
    /// ID of the primary Hero Cover image (stack_order = 0).
    pub hero_image_id: Option<i64>,
}
