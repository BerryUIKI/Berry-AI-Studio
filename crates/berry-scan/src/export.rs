use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use berry_domain::{
    ExportFormat, ExportOptions, ExportProgressEvent, ExportSidecar, ExportSummary, ImageFile,
    MetadataPrivacyMode,
};
use berry_storage::Database;
use image::ImageReader;
use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::html_showcase::{generate_html_showcase, ShowcaseItemMetadata};

/// Sanitize filename by removing invalid OS characters and trimming.
pub fn sanitize_filename_part(part: &str) -> String {
    let sanitized: String = part
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = sanitized.trim().trim_matches('.');
    if trimmed.is_empty() {
        "export".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Format an output filename based on a user-supplied template string.
pub fn format_export_filename(
    template: &str,
    file: &ImageFile,
    target_ext: &str,
    index: usize,
) -> String {
    let original_stem = Path::new(&file.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let file_id_str = file.id.map(|id| id.to_string()).unwrap_or_default();
    let rating_str = file
        .rating
        .map(|r| r.to_string())
        .unwrap_or_else(|| "0".to_string());

    let date_str = if file.modified_at > 0 {
        let secs = file.modified_at;
        let days = secs / 86400;
        let mut year = 1970;
        let mut remaining_days = days;
        loop {
            let days_in_year = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                366
            } else {
                365
            };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_per_month = [
            31,
            if is_leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1;
        for &m_days in &days_per_month {
            if remaining_days < m_days {
                break;
            }
            remaining_days -= m_days;
            month += 1;
        }
        let day = remaining_days + 1;
        format!("{year:04}{month:02}{day:02}")
    } else {
        "20260101".to_string()
    };

    let model_str = file
        .metadata
        .as_ref()
        .and_then(|m| m.model_name.as_deref())
        .unwrap_or("unknown_model");

    let seed_str = file
        .metadata
        .as_ref()
        .and_then(|m| m.seed.as_deref())
        .unwrap_or("0");

    let mut result = template.to_string();
    if result.is_empty() {
        result = "{name}".to_string();
    }

    result = result.replace("{name}", original_stem);
    result = result.replace("{filename}", original_stem);
    result = result.replace("{id}", &file_id_str);
    result = result.replace("{index}", &(index + 1).to_string());
    result = result.replace("{date}", &date_str);
    result = result.replace("{rating}", &rating_str);
    result = result.replace("{model}", model_str);
    result = result.replace("{seed}", seed_str);

    let clean_stem = sanitize_filename_part(&result);
    format!("{clean_stem}.{target_ext}")
}

/// In-memory representation of a successfully processed export item.
pub struct ProcessedExportItem {
    pub base_filename: String,
    pub image_filename: String,
    pub image_bytes: Vec<u8>,
    pub sidecar: Option<(String, Vec<u8>)>,
    pub showcase_item: Option<ShowcaseItemMetadata>,
}

/// Process a single image file according to export format, downscaling, and privacy rules.
pub fn process_single_image(
    file: &ImageFile,
    options: &ExportOptions,
    index: usize,
) -> Result<ProcessedExportItem, String> {
    let src_path = Path::new(&file.path);
    if !src_path.exists() {
        return Err(format!("Source image does not exist: {}", file.path));
    }

    let target_ext = match options.format {
        ExportFormat::Original => src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase(),
        ExportFormat::Webp => "webp".to_string(),
        ExportFormat::Jpeg => "jpg".to_string(),
        ExportFormat::Png => "png".to_string(),
    };

    let image_filename =
        format_export_filename(&options.filename_template, file, &target_ext, index);
    let base_stem = Path::new(&image_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    // Fast-path: keep original format, no resizing, no privacy stripping
    let is_fast_pass = options.format == ExportFormat::Original
        && options.privacy == MetadataPrivacyMode::KeepAll
        && options.max_edge.is_none();

    let (image_bytes, final_width, final_height) = if is_fast_pass {
        let bytes = fs::read(src_path).map_err(|e| format!("Failed to read {}: {e}", file.path))?;
        let w = file.metadata.as_ref().and_then(|m| m.width).unwrap_or(0);
        let h = file.metadata.as_ref().and_then(|m| m.height).unwrap_or(0);
        (bytes, w, h)
    } else {
        // Decode image
        let reader = ImageReader::open(src_path)
            .map_err(|e| format!("Failed to open {}: {e}", file.path))?
            .with_guessed_format()
            .map_err(|e| format!("Failed to guess format for {}: {e}", file.path))?;

        let mut img = reader
            .decode()
            .map_err(|e| format!("Failed to decode {}: {e}", file.path))?;

        // Downscale if max_edge specified
        if let Some(max_edge) = options.max_edge {
            let (w, h) = (img.width(), img.height());
            if w > max_edge || h > max_edge {
                img = img.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3);
            }
        }

        let (w, h) = (img.width(), img.height());

        // Encode to target format (pure raster encoding strips all source metadata chunks)
        let mut buffer = Vec::new();
        match options.format {
            ExportFormat::Webp => {
                img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::WebP)
                    .map_err(|e| format!("Failed to encode WebP: {e}"))?;
            }
            ExportFormat::Jpeg => {
                let rgb = img.to_rgb8();
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buffer,
                    options.quality.clamp(1, 100),
                );
                encoder
                    .encode_image(&rgb)
                    .map_err(|e| format!("Failed to encode JPEG: {e}"))?;
            }
            ExportFormat::Png | ExportFormat::Original => {
                img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
                    .map_err(|e| format!("Failed to encode PNG: {e}"))?;
            }
        }
        (buffer, w, h)
    };

    // Generate optional sidecar
    let sidecar = match options.sidecar {
        ExportSidecar::None => None,
        ExportSidecar::TextPrompt => {
            let prompt_text = file
                .metadata
                .as_ref()
                .and_then(|m| m.prompt.as_deref())
                .or_else(|| file.metadata.as_ref().and_then(|m| m.raw.as_deref()))
                .unwrap_or_default()
                .to_string();
            let sidecar_filename = format!("{base_stem}.txt");
            Some((sidecar_filename, prompt_text.into_bytes()))
        }
        ExportSidecar::JsonMetadata => {
            let json_str = file
                .metadata
                .as_ref()
                .and_then(|m| serde_json::to_string_pretty(m).ok())
                .unwrap_or_else(|| "{}".to_string());
            let sidecar_filename = format!("{base_stem}.json");
            Some((sidecar_filename, json_str.into_bytes()))
        }
    };

    let showcase_item = if options.export_html_showcase {
        let (prompt, negative_prompt, model, sampler, seed, cfg_scale, steps) =
            match options.privacy {
                MetadataPrivacyMode::KeepAll => (
                    file.metadata.as_ref().and_then(|m| m.prompt.clone()),
                    file.metadata
                        .as_ref()
                        .and_then(|m| m.negative_prompt.clone()),
                    file.metadata.as_ref().and_then(|m| m.model_name.clone()),
                    file.metadata.as_ref().and_then(|m| m.sampler.clone()),
                    file.metadata.as_ref().and_then(|m| m.seed.clone()),
                    file.metadata.as_ref().and_then(|m| m.cfg_scale),
                    file.metadata.as_ref().and_then(|m| m.steps),
                ),
                MetadataPrivacyMode::StripPromptOnly => (
                    None,
                    None,
                    file.metadata.as_ref().and_then(|m| m.model_name.clone()),
                    file.metadata.as_ref().and_then(|m| m.sampler.clone()),
                    file.metadata.as_ref().and_then(|m| m.seed.clone()),
                    file.metadata.as_ref().and_then(|m| m.cfg_scale),
                    file.metadata.as_ref().and_then(|m| m.steps),
                ),
                MetadataPrivacyMode::StripAllAiMetadata | MetadataPrivacyMode::StripAll => {
                    (None, None, None, None, None, None, None)
                }
            };

        Some(ShowcaseItemMetadata {
            filename: image_filename.clone(),
            width: final_width,
            height: final_height,
            prompt,
            negative_prompt,
            model,
            sampler,
            seed,
            cfg_scale,
            steps,
            rating: file.rating,
        })
    } else {
        None
    };

    Ok(ProcessedExportItem {
        base_filename: base_stem,
        image_filename,
        image_bytes,
        sidecar,
        showcase_item,
    })
}

/// Execute a complete batch export job with chunked concurrency and progress events.
pub fn execute_batch_export<P>(
    db: &Database,
    options: &ExportOptions,
    progress_callback: P,
) -> ExportSummary
where
    P: Fn(ExportProgressEvent) + Send + Sync + 'static,
{
    let start_time = Instant::now();
    let total = options.file_ids.len();
    let mut errors = Vec::new();
    let mut total_exported = 0;
    let mut total_bytes_written = 0u64;

    // Load file records from database
    let mut files = Vec::with_capacity(total);
    for &id in &options.file_ids {
        match db.get_file_by_id(id) {
            Ok(Some(file)) => files.push(file),
            Ok(None) => errors.push(format!("File id {id} not found in database")),
            Err(e) => errors.push(format!("Database error querying file id {id}: {e}")),
        }
    }

    let dest_path = Path::new(&options.destination_path);
    let mut zip_writer = if options.as_zip {
        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::File::create(dest_path) {
            Ok(file) => Some(ZipWriter::new(file)),
            Err(e) => {
                errors.push(format!("Failed to create output zip file: {e}"));
                return ExportSummary {
                    success: false,
                    total_exported: 0,
                    total_failed: total,
                    total_bytes_written: 0,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    output_path: options.destination_path.clone(),
                    errors,
                };
            }
        }
    } else {
        if let Err(e) = fs::create_dir_all(dest_path) {
            errors.push(format!("Failed to create destination directory: {e}"));
            return ExportSummary {
                success: false,
                total_exported: 0,
                total_failed: total,
                total_bytes_written: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                output_path: options.destination_path.clone(),
                errors,
            };
        }
        None
    };

    let zip_options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let processed_counter = AtomicUsize::new(0);
    let mut showcase_items = Vec::new();

    // Process files in bounded chunks of 16 to keep memory usage strictly bounded
    for (chunk_idx, chunk) in files.chunks(16).enumerate() {
        let chunk_offset = chunk_idx * 16;
        let results: Vec<Result<ProcessedExportItem, String>> = chunk
            .par_iter()
            .enumerate()
            .map(|(i, file)| process_single_image(file, options, chunk_offset + i))
            .collect();

        for res in results {
            let current = processed_counter.fetch_add(1, Ordering::Relaxed) + 1;
            match res {
                Ok(item) => {
                    let image_len = item.image_bytes.len() as u64;
                    if let Some(ref mut zip) = zip_writer {
                        use std::io::Write;
                        let _ = zip.start_file(&item.image_filename, zip_options);
                        let _ = zip.write_all(&item.image_bytes);
                        total_bytes_written += image_len;

                        if let Some((sidecar_name, sidecar_bytes)) = item.sidecar {
                            total_bytes_written += sidecar_bytes.len() as u64;
                            let _ = zip.start_file(&sidecar_name, zip_options);
                            let _ = zip.write_all(&sidecar_bytes);
                        }
                    } else {
                        let out_image_path = dest_path.join(&item.image_filename);
                        if let Err(e) = fs::write(&out_image_path, &item.image_bytes) {
                            errors
                                .push(format!("Failed to write {}: {e}", out_image_path.display()));
                        } else {
                            total_bytes_written += image_len;
                        }

                        if let Some((sidecar_name, sidecar_bytes)) = item.sidecar {
                            let out_sidecar_path = dest_path.join(&sidecar_name);
                            if let Ok(()) = fs::write(&out_sidecar_path, &sidecar_bytes) {
                                total_bytes_written += sidecar_bytes.len() as u64;
                            }
                        }
                    }

                    if let Some(showcase) = item.showcase_item {
                        showcase_items.push(showcase);
                    }

                    progress_callback(ExportProgressEvent {
                        current,
                        total,
                        current_filename: item.image_filename,
                    });
                    total_exported += 1;
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
    }

    if options.export_html_showcase && !showcase_items.is_empty() {
        let title = options
            .html_title
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Omera Showcase");
        let html_content = generate_html_showcase(title, &showcase_items);
        let html_bytes = html_content.as_bytes();
        let html_len = html_bytes.len() as u64;

        if let Some(ref mut zip) = zip_writer {
            use std::io::Write;
            let _ = zip.start_file("index.html", zip_options);
            let _ = zip.write_all(html_bytes);
            total_bytes_written += html_len;
        } else {
            let out_html_path = dest_path.join("index.html");
            if let Err(e) = fs::write(&out_html_path, html_bytes) {
                errors.push(format!("Failed to write index.html: {e}"));
            } else {
                total_bytes_written += html_len;
            }
        }
    }

    if let Some(zip) = zip_writer {
        if let Err(e) = zip.finish() {
            errors.push(format!("Failed to finalize zip archive: {e}"));
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let total_failed = total.saturating_sub(total_exported);

    ExportSummary {
        success: errors.is_empty(),
        total_exported,
        total_failed,
        total_bytes_written,
        duration_ms,
        output_path: options.destination_path.clone(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use berry_domain::{Container, ExtractedMetadata, MetadataFormat};
    use image::{Rgb, RgbImage};

    #[test]
    fn test_format_export_filename_substitution() {
        let file = ImageFile {
            id: Some(42),
            folder_id: 1,
            path: "C:\\AI_Images\\hero_cyberpunk.png".to_string(),
            container: Container::Png,
            size_bytes: 1024,
            modified_at: 1726000000,
            metadata: None,
            rating: Some(5),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let formatted = format_export_filename("{date}_{name}_r{rating}_{id}", &file, "webp", 0);
        assert!(formatted.starts_with("2024"));
        assert!(formatted.contains("hero_cyberpunk"));
        assert!(formatted.contains("r5_42"));
        assert!(formatted.ends_with(".webp"));
    }

    #[test]
    fn test_sanitize_filename_part() {
        let dirty = "my:bad/file*name?<test>|";
        let clean = sanitize_filename_part(dirty);
        assert_eq!(clean, "my_bad_file_name__test__");
    }

    #[test]
    fn test_process_single_image_webp_downscale_and_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("sample.png");

        // Create a test 200x100 RGB image
        let mut img = RgbImage::new(200, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 128, 0]);
        }
        img.save(&src_img_path).unwrap();

        let metadata = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("beautiful sunset, masterpiece".to_string()),
            raw: None,
            prompt: Some("beautiful sunset, masterpiece".to_string()),
            negative_prompt: Some("low quality, blurry".to_string()),
            width: Some(200),
            height: Some(100),
            seed: Some("123456789".to_string()),
            steps: Some(30),
            cfg_scale: Some(7.5),
            sampler: Some("Euler a".to_string()),
            model_name: Some("dreamshaper_v8".to_string()),
            model_hash: Some("abcdef12".to_string()),
            duration_seconds: None,
            fps: None,
            video_codec: None,
        };

        let file = ImageFile {
            id: Some(1),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: Some(metadata),
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![1],
            format: ExportFormat::Webp,
            quality: 80,
            privacy: MetadataPrivacyMode::StripAllAiMetadata,
            sidecar: ExportSidecar::TextPrompt,
            filename_template: "{model}_{id}_{name}".to_string(),
            destination_path: dir.path().join("output.zip").to_string_lossy().to_string(),
            as_zip: true,
            max_edge: Some(100),
            export_html_showcase: true,
            html_title: Some("My Test Showcase".to_string()),
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.image_filename, "dreamshaper_v8_1_sample.webp");
        assert!(!processed.image_bytes.is_empty());

        // Verify decoded dimensions were downscaled to max_edge = 100
        let decoded = ImageReader::new(Cursor::new(&processed.image_bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 50);

        // Verify sidecar
        assert!(processed.sidecar.is_some());
        let (sidecar_name, sidecar_bytes) = processed.sidecar.unwrap();
        assert_eq!(sidecar_name, "dreamshaper_v8_1_sample.txt");
        assert_eq!(
            String::from_utf8(sidecar_bytes).unwrap(),
            "beautiful sunset, masterpiece"
        );

        // Verify showcase item
        assert!(processed.showcase_item.is_some());
        let showcase = processed.showcase_item.unwrap();
        assert_eq!(showcase.filename, "dreamshaper_v8_1_sample.webp");
        assert_eq!(showcase.width, 100);
        assert_eq!(showcase.height, 50);
        // Privacy was StripAllAiMetadata so prompt is stripped
        assert!(showcase.prompt.is_none());
    }
}
