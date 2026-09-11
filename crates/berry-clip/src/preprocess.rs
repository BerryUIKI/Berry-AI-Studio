use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

/// Preprocesses an image for standard CLIP / SigLIP vision models:
/// 1. Resize and center crop (or direct resize) to `(size, size)`
/// 2. Convert to RGB float in [0.0, 1.0]
/// 3. Normalize with CLIP standard mean and std:
///    mean = [0.48145466, 0.4578275, 0.40821073]
///    std  = [0.26862954, 0.26130258, 0.27577711]
/// 4. Output shape: [1, 3, size, size] in NCHW planar layout.
#[allow(clippy::excessive_precision)]
pub fn preprocess_image(img: &DynamicImage, size: u32) -> ([usize; 4], Vec<f32>) {
    let mean = [0.48145466f32, 0.4578275f32, 0.40821073f32];
    let std = [0.26862954f32, 0.261_302_6_f32, 0.275_777_1_f32];

    // Crop or resize keeping aspect ratio if desirable, or resize_exact
    // Standard CLIP vision processors resize shorter edge to 224 then center crop 224x224.
    let (width, height) = img.dimensions();
    let cropped = if width == height {
        img.resize_exact(size, size, FilterType::Triangle)
    } else {
        let (nwidth, nheight) = if width < height {
            let nw = size;
            let nh = ((height as f32 * size as f32) / width as f32).round() as u32;
            (nw, nh.max(size))
        } else {
            let nh = size;
            let nw = ((width as f32 * size as f32) / height as f32).round() as u32;
            (nw.max(size), nh)
        };
        let resized = img.resize_exact(nwidth, nheight, FilterType::Triangle);
        let x = (nwidth.saturating_sub(size)) / 2;
        let y = (nheight.saturating_sub(size)) / 2;
        resized.crop_imm(x, y, size, size)
    };

    let rgb = cropped.to_rgb8();
    let pixel_count = (size * size) as usize;
    let mut data = vec![0.0f32; 3 * pixel_count];

    let r_offset = 0;
    let g_offset = pixel_count;
    let b_offset = 2 * pixel_count;

    for (i, pixel) in rgb.pixels().enumerate() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        data[r_offset + i] = (r - mean[0]) / std[0];
        data[g_offset + i] = (g - mean[1]) / std[1];
        data[b_offset + i] = (b - mean[2]) / std[2];
    }

    ([1, 3, size as usize, size as usize], data)
}

/// Helper to L2-normalize an embedding vector in-place or returning a new Vec.
pub fn l2_normalize(vec: &[f32]) -> Vec<f32> {
    let mut sum_sq: f64 = 0.0;
    for &v in vec {
        let vf = v as f64;
        sum_sq += vf * vf;
    }
    let norm = sum_sq.sqrt();
    if norm <= 1e-12 || !norm.is_finite() {
        return vec.to_vec();
    }
    vec.iter().map(|&v| (v as f64 / norm) as f32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn preprocess_outputs_expected_shape_and_range() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 200));
        let (shape, data) = preprocess_image(&img, 224);
        assert_eq!(shape, [1, 3, 224, 224]);
        assert_eq!(data.len(), 3 * 224 * 224);
        assert!(data.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn l2_normalize_works() {
        let v = vec![3.0, 4.0];
        let norm = l2_normalize(&v);
        assert!((norm[0] - 0.6).abs() < 1e-5);
        assert!((norm[1] - 0.8).abs() < 1e-5);
    }
}
