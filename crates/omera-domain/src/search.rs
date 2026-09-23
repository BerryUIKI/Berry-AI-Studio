use serde::{Deserialize, Serialize};

use crate::{FileSortField, ImageFile, SortDirection};

/// One bounded page of files plus the exact size of the filtered result set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePage {
    pub items: Vec<ImageFile>,
    pub total: usize,
    pub offset: usize,
    pub has_more: bool,
}

/// Criteria for filtering and querying files in the library.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchCriteria {
    /// Broad text query matching across prompt, negative_prompt, model_name, and path.
    pub text: Option<String>,
    /// Filter specifically by prompt content (partial match).
    pub prompt: Option<String>,
    /// Filter specifically by negative prompt content (partial match).
    pub negative_prompt: Option<String>,
    /// Filter by model name (partial match).
    pub model_name: Option<String>,
    /// Filter by model hash (partial match).
    pub model_hash: Option<String>,
    /// Filter by sampler name (partial match).
    pub sampler: Option<String>,
    /// Minimum generation steps.
    pub min_steps: Option<u32>,
    /// Maximum generation steps.
    pub max_steps: Option<u32>,
    /// Minimum CFG scale.
    pub min_cfg: Option<f64>,
    /// Maximum CFG scale.
    pub max_cfg: Option<f64>,
    /// Minimum user rating (1–10).
    pub min_rating: Option<u8>,
    /// Maximum user rating (1–10).
    pub max_rating: Option<u8>,
    /// Minimum aesthetic score.
    pub min_aesthetic: Option<f64>,
    /// Maximum aesthetic score.
    pub max_aesthetic: Option<f64>,
    /// Filter by favorite status.
    pub is_favorite: Option<bool>,
    /// Filter by NSFW status.
    pub is_nsfw: Option<bool>,
    /// Optional album constraint. If set, only files belonging to this album are returned.
    pub album_id: Option<i64>,
    /// Optional tag constraint. If set, only files tagged with this tag are returned.
    pub tag_id: Option<i64>,
    /// Optional folder constraint. If `None`, searches across all indexed folders.
    pub folder_id: Option<i64>,
    /// Optional folder directory path constraint (e.g. subfolder path).
    #[serde(default)]
    pub folder_path: Option<String>,
    /// Whether folder_path matches recursively (default: true).
    #[serde(default)]
    pub recursive: Option<bool>,
    /// Optional stack constraint used by filtered stack expansion.
    pub stack_id: Option<String>,
    /// Media type constraint ("all" | "image" | "video").
    #[serde(default)]
    pub media_type: Option<String>,
    /// Minimum video duration in seconds.
    #[serde(default)]
    pub min_duration: Option<f64>,
    /// Maximum video duration in seconds.
    #[serde(default)]
    pub max_duration: Option<f64>,
    /// Minimum video frame rate (fps).
    #[serde(default)]
    pub min_fps: Option<f64>,
    /// Maximum video frame rate (fps).
    #[serde(default)]
    pub max_fps: Option<f64>,
    /// Field to sort results by. Defaults to `ModifiedAt`.
    pub sort: Option<FileSortField>,
    /// Sort direction. Defaults to `Desc`.
    pub direction: Option<SortDirection>,
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// Number of records to skip (for pagination).
    pub offset: Option<usize>,
    /// Optional keyset cursor for O(1) deep-page pagination.
    #[serde(default)]
    pub cursor: Option<PageCursor>,
}

/// Keyset cursor for O(1) deep-page pagination across 500k+ assets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageCursor {
    /// Serialized sort column value (e.g. unix timestamp, rating, or byte size).
    pub sort_value: String,
    /// Tie-breaking file row ID.
    pub id: i64,
}

/// A page of results retrieved via keyset cursor pagination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorFilePage {
    pub items: Vec<ImageFile>,
    pub total: usize,
    pub next_cursor: Option<PageCursor>,
    pub prev_cursor: Option<PageCursor>,
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_criteria_serde_roundtrip() {
        let criteria = SearchCriteria {
            text: Some("cyberpunk".to_string()),
            prompt: Some("neon cityscape".to_string()),
            negative_prompt: Some("blurry".to_string()),
            model_name: Some("dreamshaper".to_string()),
            model_hash: Some("abc12345".to_string()),
            sampler: Some("Euler a".to_string()),
            min_steps: Some(20),
            max_steps: Some(50),
            min_cfg: Some(7.0),
            max_cfg: Some(12.5),
            min_rating: Some(8),
            max_rating: Some(10),
            min_aesthetic: Some(0.6),
            max_aesthetic: Some(0.95),
            is_favorite: Some(true),
            is_nsfw: Some(false),
            album_id: Some(10),
            tag_id: Some(5),
            folder_id: Some(42),
            folder_path: Some("/library/sub".to_string()),
            recursive: Some(true),
            stack_id: Some("stack-a".to_string()),
            media_type: Some("video".to_string()),
            min_duration: Some(5.0),
            max_duration: Some(30.0),
            min_fps: Some(24.0),
            max_fps: Some(60.0),
            sort: Some(FileSortField::Rating),
            direction: Some(SortDirection::Desc),
            limit: Some(100),
            offset: Some(0),
            cursor: None,
        };

        let json = serde_json::to_string(&criteria).unwrap();
        let decoded: SearchCriteria = serde_json::from_str(&json).unwrap();
        assert_eq!(criteria, decoded);
    }

    #[test]
    fn empty_criteria_defaults() {
        let criteria = SearchCriteria::default();
        assert!(criteria.text.is_none());
        assert!(criteria.prompt.is_none());
        assert!(criteria.min_rating.is_none());
        assert!(criteria.sort.is_none());
    }

    #[test]
    fn file_page_serde_roundtrip() {
        let page = FilePage {
            items: Vec::new(),
            total: 25,
            offset: 10,
            has_more: true,
        };
        let json = serde_json::to_string(&page).unwrap();
        assert_eq!(serde_json::from_str::<FilePage>(&json).unwrap(), page);
    }

    #[test]
    fn cursor_file_page_serde_roundtrip() {
        let cursor_page = CursorFilePage {
            items: Vec::new(),
            total: 500_000,
            next_cursor: Some(PageCursor {
                sort_value: "1726000000".to_string(),
                id: 489123,
            }),
            prev_cursor: None,
            has_more: true,
        };
        let json = serde_json::to_string(&cursor_page).unwrap();
        assert_eq!(
            serde_json::from_str::<CursorFilePage>(&json).unwrap(),
            cursor_page
        );
    }
}
