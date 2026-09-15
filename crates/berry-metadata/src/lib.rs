//! Metadata detection and extraction for AI-generated image formats.
//!
//! [`detect_container`] sniffs a file's container from its magic bytes.
//! [`extract_metadata`] is the format-dispatch entry point used by the scan
//! engine: given a container and a file path it returns structured metadata or
//! `None`. Embedded metadata (PNGInfo, EXIF) is tried first; when a file carries
//! none, a sibling `.txt` sidecar is used as a fallback.

use std::path::Path;

use berry_domain::{Container, ExtractedMetadata, MetadataFormat};

pub mod comfyui;
pub mod container;
pub mod easydiffusion;
pub mod exif;
pub mod fooocus;
pub mod invokeai;
pub mod lora;
pub mod novelai;
pub mod parameters;
pub mod pnginfo;
pub mod sidecar;

pub use container::detect_container;

/// Extract structured metadata from a media file, dispatching on its container.
///
/// Returns `None` when the file carries no recognizable metadata. Only image
/// containers are considered; videos and text are out of scope. Embedded
/// metadata wins; a `.txt` sidecar is only consulted as a fallback.
pub fn extract_metadata(container: Container, path: &Path) -> Option<ExtractedMetadata> {
    if !container.is_image() {
        return None;
    }
    let embedded = match container {
        Container::Png => extract_png_metadata(path),
        Container::Jpeg | Container::WebP => extract_exif_metadata(path),
        Container::Mp4 | Container::Txt => None,
    };
    embedded.or_else(|| extract_sidecar_metadata(path))
}

/// Extract metadata from a PNG file across all supported generator formats.
fn extract_png_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);
    let chunks = pnginfo::read_text_chunks_from_reader(reader).ok()?;

    // 1. Look for ComfyUI `prompt` / `workflow`
    for chunk in &chunks {
        if chunk.keyword == "prompt" || chunk.keyword == "workflow" {
            if let Some(meta) = comfyui::parse_comfyui(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 2. Look for InvokeAI chunks (`sd-metadata`, `invokeai_metadata`)
    for chunk in &chunks {
        if chunk.keyword == "sd-metadata"
            || chunk.keyword == "invokeai_metadata"
            || chunk.keyword == "invokeai"
        {
            if let Some(meta) = invokeai::parse_invokeai(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 3. Look for Stable Swarm
    for chunk in &chunks {
        if chunk.keyword == "sui_image_params" {
            if let Some(meta) = easydiffusion::parse_stableswarm(&chunk.text) {
                return Some(meta);
            }
        }
    }

    // 4. Look for NovelAI / EasyDiffusion in `Comment`
    let comment_chunk = chunks.iter().find(|c| c.keyword == "Comment");
    let description_chunk = chunks.iter().find(|c| c.keyword == "Description");
    if let Some(c) = comment_chunk {
        if let Some(meta) =
            novelai::parse_novelai(&c.text, description_chunk.map(|d| d.text.as_str()))
        {
            return Some(meta);
        }
        if let Some(meta) = easydiffusion::parse_easydiffusion(&c.text) {
            return Some(meta);
        }
        if let Some(meta) = comfyui::parse_comfyui(&c.text) {
            return Some(meta);
        }
    }

    // 5. Look for `parameters` chunk (A1111 / Fooocus / EasyDiffusion / ComfyUI)
    if let Some(param_chunk) = chunks.iter().find(|c| c.keyword == "parameters") {
        let text = &param_chunk.text;
        // Check Fooocus
        if let Some(meta) = fooocus::parse_fooocus(text) {
            return Some(meta);
        }
        // Check JSON formats (ComfyUI / EasyDiffusion)
        if text.trim_start().starts_with('{') {
            if let Some(meta) = comfyui::parse_comfyui(text) {
                return Some(meta);
            }
            if let Some(meta) = easydiffusion::parse_easydiffusion(text) {
                return Some(meta);
            }
            if let Some(meta) = novelai::parse_novelai(text, None) {
                return Some(meta);
            }
        }
        // Default to A1111
        return Some(from_parameters(text.clone()));
    }

    // 6. Description chunk alone (NovelAI / StableDiffusion fallback)
    if let Some(desc) = description_chunk {
        if !desc.text.trim().is_empty() {
            return Some(ExtractedMetadata {
                format: MetadataFormat::NovelAI,
                parameters: Some(desc.text.clone()),
                raw: Some(desc.text.clone()),
                prompt: Some(desc.text.trim().to_string()),
                negative_prompt: None,
                width: None,
                height: None,
                seed: None,
                steps: None,
                cfg_scale: None,
                sampler: None,
                model_name: None,
                model_hash: None,
            });
        }
    }

    None
}

/// Extract dimensions + generator name from a JPEG/WebP file's EXIF block.
fn extract_exif_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let info = exif::read_exif(path).ok()??;
    let software = info.software.as_deref()?;
    let format = exif::infer_format(software)?;
    Some(ExtractedMetadata {
        format,
        parameters: None,
        raw: info.software,
        prompt: None,
        negative_prompt: None,
        width: info.width,
        height: info.height,
        seed: None,
        steps: None,
        cfg_scale: None,
        sampler: None,
        model_name: None,
        model_hash: None,
    })
}

/// Read a sibling `<file>.txt` and parse it as Fooocus or A1111-style parameters.
fn extract_sidecar_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let text = sidecar::read_sidecar(path)?;
    if let Some(meta) = fooocus::parse_fooocus(&text) {
        return Some(meta);
    }
    Some(from_parameters(text))
}

/// Build [`ExtractedMetadata`] from an A1111-style parameter string.
fn from_parameters(parameters: String) -> ExtractedMetadata {
    let parsed = parameters::parse_parameters(&parameters);
    ExtractedMetadata {
        format: MetadataFormat::A1111,
        parameters: Some(parameters),
        raw: None,
        prompt: parsed.prompt,
        negative_prompt: parsed.negative_prompt,
        width: parsed.width,
        height: parsed.height,
        seed: parsed.seed,
        steps: parsed.steps,
        cfg_scale: parsed.cfg_scale,
        sampler: parsed.sampler,
        model_name: parsed.model_name,
        model_hash: parsed.model_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tex(keyword: &str, text: &str) -> Vec<u8> {
        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.extend_from_slice(text.as_bytes());
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(b"tEXt");
        out.extend_from_slice(&data);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn png_with_parameters(text: &str) -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("parameters", text));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn plain_png() -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    /// Minimal JPEG with an EXIF APP1 segment (`ComfyUI` software + dimensions).
    fn exif_jpeg() -> Vec<u8> {
        let software = "ComfyUI";
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&0x002A_u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&1u16.to_le_bytes()); // single entry: Software
        tiff.extend_from_slice(&0x0131_u16.to_le_bytes());
        tiff.extend_from_slice(&2u16.to_le_bytes());
        tiff.extend_from_slice(&(software.len() as u32 + 1).to_le_bytes());
        tiff.extend_from_slice(&26u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD offset
        tiff.extend_from_slice(software.as_bytes());
        tiff.push(0);

        let marker = b"Exif\0\0";
        let app1_len = 2 + marker.len() + tiff.len();
        let mut out = vec![0xFF, 0xD8];
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&(app1_len as u16).to_be_bytes());
        out.extend_from_slice(marker);
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    /// Helper: write `bytes` to a fresh temp subdirectory and return the path.
    fn temp_file(dir: &str, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("berry-meta-{}", std::process::id()))
            .join(dir);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn extracts_png_metadata() {
        let path = temp_file(
            "png",
            "test.png",
            &png_with_parameters(
                "a cat\nNegative prompt: blurry\nSteps: 20, Seed: 42, Size: 512x768",
            ),
        );

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a cat"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
        assert_eq!(meta.width, Some(512));
        assert_eq!(meta.height, Some(768));

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_exif_metadata_from_jpeg() {
        let path = temp_file("exif", "comfy.jpg", &exif_jpeg());
        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(meta.raw.as_deref(), Some("ComfyUI"));
        assert_eq!(meta.prompt, None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_jpeg_falls_back_to_sidecar() {
        let path = temp_file("jpeg-sidecar", "plain.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        std::fs::write(
            path.with_extension("txt"),
            "a landscape\nSteps: 12, Sampler: DPM++ 2M",
        )
        .unwrap();

        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a landscape"));
        assert_eq!(meta.steps, Some(12));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_png_falls_back_to_sidecar() {
        let path = temp_file("png-sidecar", "plain.png", &plain_png());
        std::fs::write(path.with_extension("txt"), "just a prompt").unwrap();

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("just a prompt"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn images_with_no_metadata_return_none() {
        let path = temp_file("none", "photo.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        assert_eq!(extract_metadata(Container::Jpeg, &path), None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_comfyui_png_metadata() {
        let json = r#"{"3":{"class_type":"KSampler","inputs":{"seed":123,"steps":20,"cfg":8.0,"sampler_name":"euler","positive":["4",0]}},"4":{"class_type":"CLIPTextEncode","inputs":{"text":"comfy forest"}}}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("prompt", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-comfy", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(meta.prompt.as_deref(), Some("comfy forest"));
        assert_eq!(meta.steps, Some(20));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_novelai_png_metadata() {
        let json = r#"{"prompt":"anime warrior","uc":"lowres","steps":28,"scale":6.0,"seed":42,"sampler":"k_euler"}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("Comment", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-novelai", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::NovelAI);
        assert_eq!(meta.prompt.as_deref(), Some("anime warrior"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("lowres"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_invokeai_png_metadata() {
        let json = r#"{"positive_prompt":"cyberpunk room","steps":35,"cfg_scale":7.0,"seed":999}"#;
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("sd-metadata", json));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0]);

        let path = temp_file("png-invokeai", "test.png", &out);
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::InvokeAI);
        assert_eq!(meta.prompt.as_deref(), Some("cyberpunk room"));
        assert_eq!(meta.steps, Some(35));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_fooocus_png_metadata() {
        let text = "Prompt: dragon flying over mountains\nResolution: (1024, 1024)\nSteps: 30\nBase Model: sd_xl_base.safetensors";
        let path = temp_file("png-fooocus", "test.png", &png_with_parameters(text));
        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::Fooocus);
        assert_eq!(meta.prompt.as_deref(), Some("dragon flying over mountains"));
        assert_eq!(meta.steps, Some(30));
        assert_eq!(meta.model_name.as_deref(), Some("sd_xl_base.safetensors"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn videos_are_ignored() {
        assert_eq!(
            extract_metadata(Container::Mp4, Path::new("x.mp4")),
            None,
            "mp4 metadata extraction is out of scope"
        );
    }
}
