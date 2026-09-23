use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TagsError {
    #[error("Failed to open tags CSV file: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Invalid CSV format or missing columns")]
    FormatError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum TagCategory {
    General = 0,
    Character = 4,
    Rating = 9,
    Unknown(u32),
}

impl From<u32> for TagCategory {
    fn from(val: u32) -> Self {
        match val {
            0 => TagCategory::General,
            4 => TagCategory::Character,
            9 => TagCategory::Rating,
            other => TagCategory::Unknown(other),
        }
    }
}

impl TagCategory {
    pub fn as_u32(&self) -> u32 {
        match self {
            TagCategory::General => 0,
            TagCategory::Character => 4,
            TagCategory::Rating => 9,
            TagCategory::Unknown(val) => *val,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggerTag {
    pub tag_id: u32,
    pub name: String,
    pub category: TagCategory,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagPrediction {
    pub name: String,
    pub category: TagCategory,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggerConfig {
    pub general_threshold: f32,
    pub character_threshold: f32,
    pub include_rating: bool,
    pub max_tags: usize,
}

impl Default for TaggerConfig {
    fn default() -> Self {
        Self {
            general_threshold: 0.35,
            character_threshold: 0.85,
            include_rating: false,
            max_tags: 50,
        }
    }
}

/// Parse SmilingWolf format `selected_tags.csv`.
/// Expected format: `tag_id,name,category,count`
pub fn load_tags_csv(path: &Path) -> Result<Vec<TaggerTag>, TagsError> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(file);

    let mut tags = Vec::new();
    for result in rdr.records() {
        let record = result?;
        if record.len() < 4 {
            continue;
        }
        let tag_id: u32 = record
            .get(0)
            .and_then(|s| s.parse().ok())
            .unwrap_or(tags.len() as u32);
        let name = record.get(1).unwrap_or("").trim().to_string();
        let cat_num: u32 = record.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let count: u64 = record.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

        if !name.is_empty() {
            tags.push(TaggerTag {
                tag_id,
                name,
                category: TagCategory::from(cat_num),
                count,
            });
        }
    }

    if tags.is_empty() {
        return Err(TagsError::FormatError);
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_selected_tags_csv() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "tag_id,name,category,count").unwrap();
        writeln!(tmp, "0,general,9,0").unwrap();
        writeln!(tmp, "1,1girl,0,1000").unwrap();
        writeln!(tmp, "2,hatsune_miku,4,500").unwrap();

        let tags = load_tags_csv(tmp.path()).unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "general");
        assert_eq!(tags[0].category, TagCategory::Rating);
        assert_eq!(tags[1].name, "1girl");
        assert_eq!(tags[1].category, TagCategory::General);
        assert_eq!(tags[2].name, "hatsune_miku");
        assert_eq!(tags[2].category, TagCategory::Character);
    }
}
