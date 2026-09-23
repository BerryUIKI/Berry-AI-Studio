use image::{imageops, DynamicImage, Rgb, RgbImage};

/// Preprocess an image for WD14 Tagger:
/// 1. Resize image preserving aspect ratio to fit inside `target_size x target_size`
/// 2. Pad canvas with white pixels (255, 255, 255)
/// 3. Convert RGB to BGR channel order in unnormalized float32 [0.0..255.0]
/// 4. Output either NHWC `[1, target_size, target_size, 3]` or NCHW `[1, 3, target_size, target_size]`
pub fn preprocess_image(
    img: &DynamicImage,
    target_size: u32,
    is_nchw: bool,
) -> (Vec<usize>, Vec<f32>) {
    let (orig_w, orig_h) = (img.width(), img.height());
    let max_dim = orig_w.max(orig_h);

    let scale = target_size as f32 / max_dim.max(1) as f32;
    let new_w = ((orig_w as f32 * scale).round() as u32).clamp(1, target_size);
    let new_h = ((orig_h as f32 * scale).round() as u32).clamp(1, target_size);

    let resized = img
        .resize_exact(new_w, new_h, imageops::FilterType::Triangle)
        .to_rgb8();

    // Create white canvas
    let mut canvas = RgbImage::from_pixel(target_size, target_size, Rgb([255, 255, 255]));

    // Centered padding offsets
    let pad_x = (target_size - new_w) / 2;
    let pad_y = (target_size - new_h) / 2;

    imageops::overlay(&mut canvas, &resized, pad_x as i64, pad_y as i64);

    let total_pixels = (target_size * target_size) as usize;

    if is_nchw {
        // Shape: [1, 3, target_size, target_size]
        let mut b_channel = Vec::with_capacity(total_pixels);
        let mut g_channel = Vec::with_capacity(total_pixels);
        let mut r_channel = Vec::with_capacity(total_pixels);

        for pixel in canvas.pixels() {
            r_channel.push(pixel[0] as f32);
            g_channel.push(pixel[1] as f32);
            b_channel.push(pixel[2] as f32);
        }

        let mut data = Vec::with_capacity(total_pixels * 3);
        // BGR order: B, G, R
        data.extend(b_channel);
        data.extend(g_channel);
        data.extend(r_channel);

        (vec![1, 3, target_size as usize, target_size as usize], data)
    } else {
        // Shape: [1, target_size, target_size, 3] (NHWC, default for SmilingWolf WD14)
        let mut data = Vec::with_capacity(total_pixels * 3);
        for pixel in canvas.pixels() {
            // BGR order
            data.push(pixel[2] as f32);
            data.push(pixel[1] as f32);
            data.push(pixel[0] as f32);
        }

        (vec![1, target_size as usize, target_size as usize, 3], data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn preprocess_produces_correct_nhwc_shape_and_bgr_values() {
        // 100x50 red image
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            100,
            50,
            image::Rgba([255, 0, 0, 255]),
        ));
        let (shape, data) = preprocess_image(&img, 448, false);

        assert_eq!(shape, vec![1, 448, 448, 3]);
        assert_eq!(data.len(), 448 * 448 * 3);

        // Center pixel should be red in BGR: B=0, G=0, R=255
        let center_idx = (224 * 448 + 224) * 3;
        assert_eq!(data[center_idx], 0.0); // B
        assert_eq!(data[center_idx + 1], 0.0); // G
        assert_eq!(data[center_idx + 2], 255.0); // R
    }

    #[test]
    fn preprocess_produces_correct_nchw_shape() {
        let img =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, image::Rgba([0, 255, 0, 255])));
        let (shape, data) = preprocess_image(&img, 448, true);

        assert_eq!(shape, vec![1, 3, 448, 448]);
        assert_eq!(data.len(), 3 * 448 * 448);
    }
}
