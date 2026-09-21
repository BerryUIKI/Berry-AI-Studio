//! Video metadata extraction for MP4 (ISOBMFF) and WebM (EBML) containers.
//!
//! Extracts generative workflows, prompts, and video stream properties
//! (duration, resolution, framerate, codec) from ComfyUI video exports
//! (AnimateDiff, Wan2.1, HunyuanVideo, CogVideoX, LTX-Video, SVD).

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use berry_domain::{ExtractedMetadata, MetadataFormat};

use crate::comfyui;
use crate::extract_sidecar_metadata;

/// Video track and container properties extracted from media headers.
#[derive(Debug, Default, Clone)]
pub struct VideoStreamInfo {
    pub duration_seconds: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub codec: Option<String>,
}

/// Extract metadata from an MP4 file.
pub fn extract_mp4_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    let (mut stream_info, embedded_json) = parse_mp4_stream(&mut reader).unwrap_or_default();

    // 1. Try embedded workflow / prompt JSON
    let mut meta = embedded_json
        .as_deref()
        .and_then(comfyui::parse_comfyui)
        .or_else(|| {
            // Check sibling sidecar fallback
            extract_sidecar_metadata(path)
        });

    // If neither embedded nor sidecar metadata gave prompt parameters,
    // but we extracted valid video stream info (duration/resolution), create a basic record.
    if meta.is_none() && (stream_info.duration_seconds.is_some() || stream_info.width.is_some()) {
        meta = Some(ExtractedMetadata {
            format: MetadataFormat::ComfyUI,
            parameters: None,
            raw: None,
            prompt: None,
            negative_prompt: None,
            width: stream_info.width,
            height: stream_info.height,
            seed: None,
            steps: None,
            cfg_scale: None,
            sampler: None,
            model_name: None,
            model_hash: None,
            duration_seconds: stream_info.duration_seconds,
            fps: stream_info.fps,
            video_codec: stream_info.codec.clone(),
        });
    }

    if let Some(ref mut m) = meta {
        if m.duration_seconds.is_none() {
            m.duration_seconds = stream_info.duration_seconds;
        }
        if m.fps.is_none() {
            m.fps = stream_info.fps;
        }
        if m.video_codec.is_none() {
            m.video_codec = stream_info.codec.take();
        }
        if m.width.is_none() {
            m.width = stream_info.width;
        }
        if m.height.is_none() {
            m.height = stream_info.height;
        }
    }

    meta
}

/// Extract metadata from a WebM file.
pub fn extract_webm_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    let (mut stream_info, embedded_json) = parse_webm_stream(&mut reader).unwrap_or_default();

    let mut meta = embedded_json
        .as_deref()
        .and_then(comfyui::parse_comfyui)
        .or_else(|| extract_sidecar_metadata(path));

    if meta.is_none() && (stream_info.duration_seconds.is_some() || stream_info.width.is_some()) {
        meta = Some(ExtractedMetadata {
            format: MetadataFormat::ComfyUI,
            parameters: None,
            raw: None,
            prompt: None,
            negative_prompt: None,
            width: stream_info.width,
            height: stream_info.height,
            seed: None,
            steps: None,
            cfg_scale: None,
            sampler: None,
            model_name: None,
            model_hash: None,
            duration_seconds: stream_info.duration_seconds,
            fps: stream_info.fps,
            video_codec: stream_info.codec.clone(),
        });
    }

    if let Some(ref mut m) = meta {
        if m.duration_seconds.is_none() {
            m.duration_seconds = stream_info.duration_seconds;
        }
        if m.fps.is_none() {
            m.fps = stream_info.fps;
        }
        if m.video_codec.is_none() {
            m.video_codec = stream_info.codec.take();
        }
        if m.width.is_none() {
            m.width = stream_info.width;
        }
        if m.height.is_none() {
            m.height = stream_info.height;
        }
    }

    meta
}

// ---------------------------------------------------------------------------
// MP4 ISOBMFF Box Parser
// ---------------------------------------------------------------------------

fn parse_mp4_stream<R: Read + Seek>(
    reader: &mut R,
) -> std::io::Result<(VideoStreamInfo, Option<String>)> {
    let mut stream_info = VideoStreamInfo::default();
    let mut embedded_text: Option<String> = None;

    let file_len = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    let mut pos = 0u64;
    while pos + 8 <= file_len {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }

        let size32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];

        let (box_size, header_len) = if size32 == 1 {
            let mut ext = [0u8; 8];
            if reader.read_exact(&mut ext).is_err() {
                break;
            }
            (u64::from_be_bytes(ext), 16u64)
        } else if size32 == 0 {
            (file_len - pos, 8u64)
        } else {
            (size32, 8u64)
        };

        if box_size < header_len {
            break;
        }

        if box_type == b"moov" {
            let moov_end = pos + box_size;
            parse_moov(
                reader,
                pos + header_len,
                moov_end,
                &mut stream_info,
                &mut embedded_text,
            )?;
            break;
        }

        pos += box_size;
    }

    Ok((stream_info, embedded_text))
}

fn parse_moov<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }

        let size32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];

        let (box_size, header_len) = if size32 == 1 {
            let mut ext = [0u8; 8];
            if reader.read_exact(&mut ext).is_err() {
                break;
            }
            (u64::from_be_bytes(ext), 16u64)
        } else if size32 == 0 {
            (end - pos, 8u64)
        } else {
            (size32, 8u64)
        };

        if box_size < header_len || pos + box_size > end {
            break;
        }

        match box_type {
            b"mvhd" => {
                // Parse Movie Header Box for duration
                let data_len = (box_size - header_len).min(32) as usize;
                let mut data = vec![0u8; data_len];
                if reader.read_exact(&mut data).is_ok() && !data.is_empty() {
                    let version = data[0];
                    if version == 0 && data.len() >= 20 {
                        let timescale =
                            u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as f64;
                        let duration =
                            u32::from_be_bytes([data[16], data[17], data[18], data[19]]) as f64;
                        if timescale > 0.0 {
                            stream_info.duration_seconds = Some(duration / timescale);
                        }
                    } else if version == 1 && data.len() >= 32 {
                        let timescale =
                            u32::from_be_bytes([data[20], data[21], data[22], data[23]]) as f64;
                        let duration = u64::from_be_bytes([
                            data[24], data[25], data[26], data[27], data[28], data[29], data[30],
                            data[31],
                        ]) as f64;
                        if timescale > 0.0 {
                            stream_info.duration_seconds = Some(duration / timescale);
                        }
                    }
                }
            }
            b"trak" => {
                // Parse Track Box for width, height, and codec
                parse_trak(reader, pos + header_len, pos + box_size, stream_info)?;
            }
            b"udta" => {
                // User Data Box: look for meta/ilst or custom json
                parse_udta(
                    reader,
                    pos + header_len,
                    pos + box_size,
                    stream_info,
                    embedded_text,
                )?;
            }
            _ => {}
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_trak<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        match box_type {
            b"tkhd" => {
                // Track header: width and height are 16.16 fixed point numbers at the end of tkhd (84 or 96 bytes)
                let len = (box_size - 8) as usize;
                if len >= 80 {
                    let mut data = vec![0u8; len];
                    if reader.read_exact(&mut data).is_ok() {
                        let w_offset = len - 8;
                        let h_offset = len - 4;
                        let w = u32::from_be_bytes([
                            data[w_offset],
                            data[w_offset + 1],
                            data[w_offset + 2],
                            data[w_offset + 3],
                        ]) >> 16;
                        let h = u32::from_be_bytes([
                            data[h_offset],
                            data[h_offset + 1],
                            data[h_offset + 2],
                            data[h_offset + 3],
                        ]) >> 16;
                        if w > 0 && h > 0 && stream_info.width.is_none() {
                            stream_info.width = Some(w);
                            stream_info.height = Some(h);
                        }
                    }
                }
            }
            b"mdia" => {
                parse_mdia(reader, pos + 8, pos + box_size, stream_info)?;
            }
            _ => {}
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_mdia<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        if box_type == b"minf" {
            parse_minf(reader, pos + 8, pos + box_size, stream_info)?;
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_minf<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        if box_type == b"stbl" {
            parse_stbl(reader, pos + 8, pos + box_size, stream_info)?;
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_stbl<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        if box_type == b"stsd" {
            // Sample Description Box (stsd)
            // 4 bytes version/flags, 4 bytes entry count, followed by sample entries
            let mut buf = vec![0u8; (box_size - 8).min(64) as usize];
            if reader.read_exact(&mut buf).is_ok() && buf.len() >= 16 {
                // First sample entry format box type is at bytes 12..16
                let codec_tag = &buf[12..16];
                if stream_info.codec.is_none() {
                    let codec_name = match codec_tag {
                        b"avc1" => "h264",
                        b"hev1" | b"hvc1" => "hevc",
                        b"vp09" => "vp9",
                        b"av01" => "av1",
                        _ => std::str::from_utf8(codec_tag).unwrap_or("unknown"),
                    };
                    stream_info.codec = Some(codec_name.to_string());
                }
            }
        } else if box_type == b"stts" {
            // Time-to-sample box: calculate average frame rate (fps)
            let mut buf = vec![0u8; (box_size - 8).min(32) as usize];
            if reader.read_exact(&mut buf).is_ok() && buf.len() >= 16 {
                // version (4 bytes), entry_count (4 bytes), sample_count (4 bytes), sample_delta (4 bytes)
                let sample_count = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]) as f64;
                let sample_delta = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]) as f64;
                if sample_count > 0.0 && sample_delta > 0.0 {
                    if let Some(dur) = stream_info.duration_seconds {
                        if dur > 0.0 {
                            let calculated_fps = (sample_count / dur).round();
                            if (1.0..=240.0).contains(&calculated_fps) {
                                stream_info.fps = Some(calculated_fps);
                            }
                        }
                    }
                }
            }
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_udta<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    _stream_info: &mut VideoStreamInfo,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        if box_type == b"meta" {
            // Note: In MP4 'meta' has a 4-byte version/flags header before child boxes
            parse_meta_box(reader, pos + 12, pos + box_size, embedded_text)?;
        } else if box_type == b"comf" {
            // ComfyUI direct box payload
            let len = (box_size - 8) as usize;
            let mut buf = vec![0u8; len];
            if reader.read_exact(&mut buf).is_ok() {
                if let Ok(s) = std::str::from_utf8(&buf) {
                    let trimmed = s.trim();
                    if trimmed.starts_with('{') && trimmed.ends_with('}') {
                        *embedded_text = Some(trimmed.to_string());
                        return Ok(());
                    }
                }
            }
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_meta_box<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let box_type = &header[4..8];
        if box_size < 8 || pos + box_size > end {
            break;
        }

        if box_type == b"ilst" {
            parse_ilst(reader, pos + 8, pos + box_size, embedded_text)?;
            if embedded_text.is_some() {
                return Ok(());
            }
        }

        pos += box_size;
    }
    Ok(())
}

fn parse_ilst<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos + 8 <= end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut header = [0u8; 8];
        if reader.read_exact(&mut header).is_err() {
            break;
        }
        let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        if box_size < 8 || pos + box_size > end {
            break;
        }

        // Inside each item box (e.g. \xa9cmt, desc, ----), look for 'data' child box
        let item_end = pos + box_size;
        let mut child_pos = pos + 8;
        while child_pos + 8 <= item_end {
            reader.seek(SeekFrom::Start(child_pos))?;
            let mut child_hdr = [0u8; 8];
            if reader.read_exact(&mut child_hdr).is_err() {
                break;
            }
            let child_size =
                u32::from_be_bytes([child_hdr[0], child_hdr[1], child_hdr[2], child_hdr[3]]) as u64;
            let child_type = &child_hdr[4..8];
            if child_size < 8 || child_pos + child_size > item_end {
                break;
            }

            if child_type == b"data" {
                // data box: 8 bytes box header + 4 bytes flags/type + 4 bytes locale + payload
                if child_size > 16 {
                    let payload_len = (child_size - 16) as usize;
                    reader.seek(SeekFrom::Start(child_pos + 16))?;
                    let mut payload = vec![0u8; payload_len];
                    if reader.read_exact(&mut payload).is_ok() {
                        if let Ok(s) = std::str::from_utf8(&payload) {
                            let trimmed = s.trim();
                            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                                *embedded_text = Some(trimmed.to_string());
                                return Ok(());
                            }
                        }
                    }
                }
            }

            child_pos += child_size;
        }

        pos += box_size;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// WebM EBML Parser
// ---------------------------------------------------------------------------

fn parse_webm_stream<R: Read + Seek>(
    reader: &mut R,
) -> std::io::Result<(VideoStreamInfo, Option<String>)> {
    let mut stream_info = VideoStreamInfo::default();
    let mut embedded_text: Option<String> = None;

    let file_len = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    // Verify EBML Header
    let (elem_id, hdr_size, _) = read_ebml_element_header(reader)?;
    if elem_id != 0x1A45DFA3 {
        return Ok((stream_info, embedded_text));
    }
    reader.seek(SeekFrom::Current(hdr_size as i64))?;

    // Parse Segment
    while let Ok((id, size, _)) = read_ebml_element_header(reader) {
        let cur_pos = reader.stream_position()?;
        if id == 0x18538067 {
            // Segment element
            let seg_end = if size == 0x00FFFFFFFFFFFFFF {
                file_len
            } else {
                cur_pos + size
            };
            parse_ebml_segment(
                reader,
                cur_pos,
                seg_end,
                &mut stream_info,
                &mut embedded_text,
            )?;
            break;
        } else {
            reader.seek(SeekFrom::Start(cur_pos + size))?;
        }
    }

    Ok((stream_info, embedded_text))
}

fn parse_ebml_segment<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    let mut timecode_scale = 1_000_000.0; // default 1ms

    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;
        let next_pos = data_start + size;

        match id {
            0x1549A966 => {
                // Info element
                parse_ebml_info(
                    reader,
                    data_start,
                    next_pos,
                    &mut timecode_scale,
                    stream_info,
                )?;
            }
            0x1654AE6B => {
                // Tracks element
                parse_ebml_tracks(reader, data_start, next_pos, stream_info)?;
            }
            0x1254C367 => {
                // Tags element
                parse_ebml_tags(reader, data_start, next_pos, embedded_text)?;
            }
            0x1F43B675 if embedded_text.is_some() && stream_info.duration_seconds.is_some() => {
                // Cluster (video frames begin) - if we already parsed tags/info we can finish early
                break;
            }
            _ => {}
        }

        pos = next_pos;
    }
    Ok(())
}

fn parse_ebml_info<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    timecode_scale: &mut f64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0x2AD7B1 {
            // TimestampScale (uint, nanoseconds)
            if size <= 8 {
                let val = read_ebml_uint(reader, size)?;
                *timecode_scale = val as f64;
            }
        } else if id == 0x4489 {
            // Duration (float)
            if size == 4 {
                let mut buf = [0u8; 4];
                if reader.read_exact(&mut buf).is_ok() {
                    let d = f32::from_be_bytes(buf) as f64;
                    stream_info.duration_seconds = Some((d * *timecode_scale) / 1_000_000_000.0);
                }
            } else if size == 8 {
                let mut buf = [0u8; 8];
                if reader.read_exact(&mut buf).is_ok() {
                    let d = f64::from_be_bytes(buf);
                    stream_info.duration_seconds = Some((d * *timecode_scale) / 1_000_000_000.0);
                }
            }
        }

        pos = data_start + size;
    }
    Ok(())
}

fn parse_ebml_tracks<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0xAE {
            // TrackEntry
            parse_ebml_track_entry(reader, data_start, data_start + size, stream_info)?;
        }

        pos = data_start + size;
    }
    Ok(())
}

fn parse_ebml_track_entry<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0x86 {
            // CodecID (string)
            if size < 64 {
                let mut buf = vec![0u8; size as usize];
                if reader.read_exact(&mut buf).is_ok() {
                    let s = String::from_utf8_lossy(&buf);
                    let clean = match s.trim() {
                        "V_VP8" => "vp8",
                        "V_VP9" => "vp9",
                        "V_AV1" => "av1",
                        other => other,
                    };
                    stream_info.codec = Some(clean.to_string());
                }
            }
        } else if id == 0xE0 {
            // Video element
            parse_ebml_video(reader, data_start, data_start + size, stream_info)?;
        }

        pos = data_start + size;
    }
    Ok(())
}

fn parse_ebml_video<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    stream_info: &mut VideoStreamInfo,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0xB0 {
            // PixelWidth
            if size <= 4 {
                stream_info.width = Some(read_ebml_uint(reader, size)? as u32);
            }
        } else if id == 0xBA {
            // PixelHeight
            if size <= 4 {
                stream_info.height = Some(read_ebml_uint(reader, size)? as u32);
            }
        } else if id == 0x2383E3 {
            // FrameRate (float)
            if size == 4 {
                let mut buf = [0u8; 4];
                if reader.read_exact(&mut buf).is_ok() {
                    stream_info.fps = Some(f32::from_be_bytes(buf) as f64);
                }
            } else if size == 8 {
                let mut buf = [0u8; 8];
                if reader.read_exact(&mut buf).is_ok() {
                    stream_info.fps = Some(f64::from_be_bytes(buf));
                }
            }
        }

        pos = data_start + size;
    }
    Ok(())
}

fn parse_ebml_tags<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0x7373 {
            // Tag element
            parse_ebml_single_tag(reader, data_start, data_start + size, embedded_text)?;
            if embedded_text.is_some() {
                return Ok(());
            }
        }

        pos = data_start + size;
    }
    Ok(())
}

fn parse_ebml_single_tag<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    embedded_text: &mut Option<String>,
) -> std::io::Result<()> {
    let mut pos = start;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let (id, size, hdr_bytes) = match read_ebml_element_header(reader) {
            Ok(v) => v,
            Err(_) => break,
        };
        let data_start = pos + hdr_bytes as u64;

        if id == 0x67C8 {
            // SimpleTag element
            let mut tag_name = String::new();
            let mut tag_string = String::new();

            let mut tag_pos = data_start;
            let tag_end = data_start + size;
            while tag_pos < tag_end {
                reader.seek(SeekFrom::Start(tag_pos))?;
                let (tid, tsize, thdr) = match read_ebml_element_header(reader) {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let tdata_start = tag_pos + thdr as u64;

                if tid == 0x45A3 && tsize < 256 {
                    // TagName
                    let mut buf = vec![0u8; tsize as usize];
                    if reader.read_exact(&mut buf).is_ok() {
                        tag_name = String::from_utf8_lossy(&buf).to_uppercase();
                    }
                } else if tid == 0x4487 {
                    // TagString
                    let mut buf = vec![0u8; tsize as usize];
                    if reader.read_exact(&mut buf).is_ok() {
                        tag_string = String::from_utf8_lossy(&buf).to_string();
                    }
                }

                tag_pos = tdata_start + tsize;
            }

            let valid_tag = tag_name.contains("COMMENT")
                || tag_name.contains("DESCRIPTION")
                || tag_name.contains("PROMPT")
                || tag_name.contains("WORKFLOW");

            let trimmed = tag_string.trim();
            if valid_tag && trimmed.starts_with('{') && trimmed.ends_with('}') {
                *embedded_text = Some(trimmed.to_string());
                return Ok(());
            }
        }

        pos = data_start + size;
    }
    Ok(())
}

fn read_ebml_element_header<R: Read>(reader: &mut R) -> std::io::Result<(u32, u64, usize)> {
    let mut first_byte = [0u8; 1];
    reader.read_exact(&mut first_byte)?;

    let b = first_byte[0];
    let (id_len, id_mask) = if b & 0x80 != 0 {
        (1, 0xFF)
    } else if b & 0x40 != 0 {
        (2, 0xFF)
    } else if b & 0x20 != 0 {
        (3, 0xFF)
    } else if b & 0x10 != 0 {
        (4, 0xFF)
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid EBML ID",
        ));
    };

    let mut id_val = (b as u32) & id_mask;
    for _ in 1..id_len {
        let mut next = [0u8; 1];
        reader.read_exact(&mut next)?;
        id_val = (id_val << 8) | (next[0] as u32);
    }

    // Read size VINT
    let mut size_first = [0u8; 1];
    reader.read_exact(&mut size_first)?;
    let sb = size_first[0];
    let (size_len, size_mask) = if sb & 0x80 != 0 {
        (1, 0x7F)
    } else if sb & 0x40 != 0 {
        (2, 0x3F)
    } else if sb & 0x20 != 0 {
        (3, 0x1F)
    } else if sb & 0x10 != 0 {
        (4, 0x0F)
    } else if sb & 0x08 != 0 {
        (5, 0x07)
    } else if sb & 0x04 != 0 {
        (6, 0x03)
    } else if sb & 0x02 != 0 {
        (7, 0x01)
    } else if sb & 0x01 != 0 {
        (8, 0x00)
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid EBML size VINT",
        ));
    };

    let mut size_val = (sb as u64) & size_mask;
    for _ in 1..size_len {
        let mut next = [0u8; 1];
        reader.read_exact(&mut next)?;
        size_val = (size_val << 8) | (next[0] as u64);
    }

    Ok((id_val, size_val, id_len + size_len))
}

fn read_ebml_uint<R: Read>(reader: &mut R, size: u64) -> std::io::Result<u64> {
    let mut val = 0u64;
    for _ in 0..size {
        let mut b = [0u8; 1];
        reader.read_exact(&mut b)?;
        val = (val << 8) | (b[0] as u64);
    }
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_mp4_with_embedded_comfyui_metadata() {
        let mut tmp = NamedTempFile::new().unwrap();

        let prompt_json = r#"{"prompt":{"3":{"class_type":"KSampler","inputs":{"cfg":7.5,"seed":987654,"steps":25,"sampler_name":"euler","scheduler":"normal","positive":["4",0]}},"4":{"class_type":"CLIPTextEncode","inputs":{"text":"cinematic dragon soaring through storm clouds"}},"5":{"class_type":"CheckpointLoaderSimple","inputs":{"ckpt_name":"wan2.1_t2v_14b.safetensors"}},"6":{"class_type":"EmptyLatentImage","inputs":{"width":1280,"height":720}}}}"#;

        // Build mock MP4 with ftyp + moov + udta + comf
        let mut mp4_data = Vec::new();
        // ftyp box (16 bytes)
        mp4_data.extend_from_slice(&16u32.to_be_bytes());
        mp4_data.extend_from_slice(b"ftypmp42");
        mp4_data.extend_from_slice(&[0, 0, 0, 0]);

        // Build udta box with comf box
        let comf_len = (8 + prompt_json.len()) as u32;
        let mut comf_box = Vec::new();
        comf_box.extend_from_slice(&comf_len.to_be_bytes());
        comf_box.extend_from_slice(b"comf");
        comf_box.extend_from_slice(prompt_json.as_bytes());

        let udta_len = (8 + comf_box.len()) as u32;
        let mut udta_box = Vec::new();
        udta_box.extend_from_slice(&udta_len.to_be_bytes());
        udta_box.extend_from_slice(b"udta");
        udta_box.extend_from_slice(&comf_box);

        // mvhd box: timescale = 1000, duration = 5000 (5.0 seconds)
        let mut mvhd_data = vec![0u8; 24];
        // timescale at offset 12:
        mvhd_data[12..16].copy_from_slice(&1000u32.to_be_bytes());
        // duration at offset 16:
        mvhd_data[16..20].copy_from_slice(&5000u32.to_be_bytes());

        let mvhd_len = (8 + mvhd_data.len()) as u32;
        let mut mvhd_box = Vec::new();
        mvhd_box.extend_from_slice(&mvhd_len.to_be_bytes());
        mvhd_box.extend_from_slice(b"mvhd");
        mvhd_box.extend_from_slice(&mvhd_data);

        // moov box
        let moov_len = (8 + mvhd_box.len() + udta_box.len()) as u32;
        let mut moov_box = Vec::new();
        moov_box.extend_from_slice(&moov_len.to_be_bytes());
        moov_box.extend_from_slice(b"moov");
        moov_box.extend_from_slice(&mvhd_box);
        moov_box.extend_from_slice(&udta_box);

        mp4_data.extend_from_slice(&moov_box);
        tmp.write_all(&mp4_data).unwrap();

        let meta = extract_mp4_metadata(tmp.path()).expect("extracted mp4 metadata");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(
            meta.prompt.as_deref(),
            Some("cinematic dragon soaring through storm clouds")
        );
        assert_eq!(
            meta.model_name.as_deref(),
            Some("wan2.1_t2v_14b.safetensors")
        );
        assert_eq!(meta.steps, Some(25));
        assert_eq!(meta.cfg_scale, Some(7.5));
        assert_eq!(meta.seed.as_deref(), Some("987654"));
        assert_eq!(meta.width, Some(1280));
        assert_eq!(meta.height, Some(720));
        assert_eq!(meta.duration_seconds, Some(5.0));
    }

    #[test]
    fn mp4_falls_back_to_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let mp4_path = dir.path().join("video.mp4");
        let txt_path = dir.path().join("video.txt");

        std::fs::write(&mp4_path, b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00").unwrap();
        std::fs::write(
            &txt_path,
            "a futuristic cyberpunk robot walking in rain\nNegative prompt: blurry, deformed\nSteps: 30, Sampler: DPM++ 2M, CFG scale: 7, Seed: 12345",
        )
        .unwrap();

        let meta = extract_mp4_metadata(&mp4_path).expect("sidecar extracted");
        assert_eq!(
            meta.prompt.as_deref(),
            Some("a futuristic cyberpunk robot walking in rain")
        );
        assert_eq!(meta.steps, Some(30));
        assert_eq!(meta.seed.as_deref(), Some("12345"));
    }
}
