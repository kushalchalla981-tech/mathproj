//! Orientation detection and correction

use crate::eigen_analysis::{analyze_image, EigenAnalysisResult};
use crate::error::{CompressionError, Result};
use crate::image::ImageData;

/// Orientation method used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrientationMethod {
    /// Orientation determined from PCA principal axis
    #[default]
    Pca,
    /// Orientation from EXIF data
    Exif,
    /// No orientation correction applied
    None,
}

impl OrientationMethod {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pca => "pca",
            Self::Exif => "exif",
            Self::None => "none",
        }
    }
}

/// Orientation correction parameters
#[derive(Debug, Clone, Copy)]
pub struct OrientationParams {
    /// Whether to use PCA-based orientation
    pub use_pca: bool,
    /// Whether to use EXIF orientation
    pub use_exif: bool,
    /// Confidence threshold for PCA (0.0-1.0)
    pub pca_confidence_threshold: f32,
}

impl Default for OrientationParams {
    fn default() -> Self {
        Self {
            use_pca: true,
            use_exif: true,
            pca_confidence_threshold: 0.6,
        }
    }
}

/// Detect and correct image orientation
///
/// Returns (corrected_image, method_used)
pub fn correct_orientation(
    image: &ImageData,
    mode: crate::pca::OrientationMode,
) -> Result<(ImageData, OrientationMethod)> {
    match mode {
        crate::pca::OrientationMode::Disabled => {
            Ok((image.clone(), OrientationMethod::None))
        }
        crate::pca::OrientationMode::Exif => {
            if let Some(orientation) = image.exif_orientation {
                let corrected = apply_exif_orientation(image, orientation)?;
                Ok((corrected, OrientationMethod::Exif))
            } else {
                // No EXIF orientation, assume normal
                Ok((image.clone(), OrientationMethod::None))
            }
        }
        crate::pca::OrientationMode::Auto => {
            // Try PCA first using the eigen analysis module
            match analyze_image(image) {
                Ok(eigen_result) => {
                    if eigen_result.confidence >= 0.6 {
                        // Use PCA orientation
                        let corrected = rotate_image(image, eigen_result.recommended_rotation)?;
                        Ok((corrected, OrientationMethod::Pca))
                    } else if let Some(exif_orientation) = image.exif_orientation {
                        // Fall back to EXIF
                        let corrected = apply_exif_orientation(image, exif_orientation)?;
                        Ok((corrected, OrientationMethod::Exif))
                    } else {
                        // No correction needed
                        Ok((image.clone(), OrientationMethod::None))
                    }
                }
                Err(_) => {
                    // PCA failed, try EXIF fallback
                    if let Some(exif_orientation) = image.exif_orientation {
                        let corrected = apply_exif_orientation(image, exif_orientation)?;
                        Ok((corrected, OrientationMethod::Exif))
                    } else {
                        Ok((image.clone(), OrientationMethod::None))
                    }
                }
            }
        }
    }
}

/// Detect orientation using eigen analysis (returns full eigen result)
pub fn detect_orientation_with_eigen(image: &ImageData) -> Result<EigenAnalysisResult> {
    analyze_image(image)
}



/// Apply EXIF orientation
fn apply_exif_orientation(image: &ImageData, orientation: u8) -> Result<ImageData> {
    match orientation {
        1 => Ok(image.clone()), // Normal
        2 => flip_horizontal(image),
        3 => rotate_180(image),
        4 => flip_vertical(image),
        5 => transpose(image),
        6 => rotate_90(image),
        7 => transpose_then_flip(image),
        8 => rotate_270(image),
        _ => Ok(image.clone()), // Unknown, assume normal
    }
}

/// Rotate image 90 degrees clockwise
fn rotate_90(image: &ImageData) -> Result<ImageData> {
    let new_width = image.height;
    let new_height = image.width;

    let mut new_rgb = vec![0.0f32; (new_width * new_height * 3) as usize];
    let mut new_alpha: Option<Vec<f32>> = image.alpha_data.as_ref()
        .map(|_| vec![0.0f32; (new_width * new_height) as usize]);

    for y in 0..image.height {
        for x in 0..image.width {
            // Original index
            let old_idx = ((y * image.width + x) * 3) as usize;
            // New position: (height - 1 - y, x) becomes (x, y)
            let new_x = image.height - 1 - y;
            let new_y = x;
            let new_idx = ((new_y * new_width + new_x) * 3) as usize;

            new_rgb[new_idx] = image.rgb_data[old_idx];
            new_rgb[new_idx + 1] = image.rgb_data[old_idx + 1];
            new_rgb[new_idx + 2] = image.rgb_data[old_idx + 2];

            if let (Some(ref old_alpha), Some(ref mut new_alpha)) = (&image.alpha_data, &mut new_alpha
            ) {
                let old_a_idx = (y * image.width + x) as usize;
                let new_a_idx = (new_y * new_width + new_x) as usize;
                new_alpha[new_a_idx] = old_alpha[old_a_idx];
            }
        }
    }

    let mut result = if let Some(alpha) = new_alpha {
        ImageData::with_alpha(new_width, new_height, new_rgb, alpha)?
    } else {
        ImageData::new(new_width, new_height, new_rgb)?
    };

    result.color_space = image.color_space;
    result.source_path = image.source_path.clone();
    result.exif_orientation = Some(1); // Now normal orientation

    Ok(result)
}

/// Rotate image 180 degrees
fn rotate_180(image: &ImageData) -> Result<ImageData> {
    let mut new_rgb = vec![0.0f32; image.rgb_data.len()];
    let mut new_alpha: Option<Vec<f32>> = image.alpha_data.as_ref()
        .map(|a| vec![0.0f32; a.len()]);

    for y in 0..image.height {
        for x in 0..image.width {
            let old_idx = ((y * image.width + x) * 3) as usize;
            let new_y = image.height - 1 - y;
            let new_x = image.width - 1 - x;
            let new_idx = ((new_y * image.width + new_x) * 3) as usize;

            new_rgb[new_idx] = image.rgb_data[old_idx];
            new_rgb[new_idx + 1] = image.rgb_data[old_idx + 1];
            new_rgb[new_idx + 2] = image.rgb_data[old_idx + 2];

            if let (Some(ref old_alpha), Some(ref mut new_alpha)) = (&image.alpha_data, &mut new_alpha
            ) {
                let old_a_idx = (y * image.width + x) as usize;
                let new_a_idx = (new_y * image.width + new_x) as usize;
                new_alpha[new_a_idx] = old_alpha[old_a_idx];
            }
        }
    }

    let mut result = if let Some(alpha) = new_alpha {
        ImageData::with_alpha(image.width, image.height, new_rgb, alpha)?
    } else {
        ImageData::new(image.width, image.height, new_rgb)?
    };

    result.color_space = image.color_space;
    result.source_path = image.source_path.clone();
    result.exif_orientation = Some(1);

    Ok(result)
}

/// Rotate image 270 degrees clockwise (or 90 CCW)
fn rotate_270(image: &ImageData) -> Result<ImageData> {
    // Three 90-degree rotations
    let r90 = rotate_90(image)?;
    let r180 = rotate_90(&r90)?;
    rotate_90(&r180)
}

/// Flip image horizontally
fn flip_horizontal(image: &ImageData) -> Result<ImageData> {
    let mut new_rgb = vec![0.0f32; image.rgb_data.len()];
    let mut new_alpha: Option<Vec<f32>> = image.alpha_data.as_ref()
        .map(|a| vec![0.0f32; a.len()]);

    for y in 0..image.height {
        for x in 0..image.width {
            let old_idx = ((y * image.width + x) * 3) as usize;
            let new_x = image.width - 1 - x;
            let new_idx = ((y * image.width + new_x) * 3) as usize;

            new_rgb[new_idx] = image.rgb_data[old_idx];
            new_rgb[new_idx + 1] = image.rgb_data[old_idx + 1];
            new_rgb[new_idx + 2] = image.rgb_data[old_idx + 2];

            if let (Some(ref old_alpha), Some(ref mut new_alpha)) = (&image.alpha_data, &mut new_alpha
            ) {
                let old_a_idx = (y * image.width + x) as usize;
                let new_a_idx = (y * image.width + new_x) as usize;
                new_alpha[new_a_idx] = old_alpha[old_a_idx];
            }
        }
    }

    let mut result = if let Some(alpha) = new_alpha {
        ImageData::with_alpha(image.width, image.height, new_rgb, alpha)?
    } else {
        ImageData::new(image.width, image.height, new_rgb)?
    };

    result.color_space = image.color_space;
    result.source_path = image.source_path.clone();
    result.exif_orientation = Some(1);

    Ok(result)
}

/// Flip image vertically
fn flip_vertical(image: &ImageData) -> Result<ImageData> {
    let mut new_rgb = vec![0.0f32; image.rgb_data.len()];
    let mut new_alpha: Option<Vec<f32>> = image.alpha_data.as_ref()
        .map(|a| vec![0.0f32; a.len()]);

    for y in 0..image.height {
        let new_y = image.height - 1 - y;
        for x in 0..image.width {
            let old_idx = ((y * image.width + x) * 3) as usize;
            let new_idx = ((new_y * image.width + x) * 3) as usize;

            new_rgb[new_idx] = image.rgb_data[old_idx];
            new_rgb[new_idx + 1] = image.rgb_data[old_idx + 1];
            new_rgb[new_idx + 2] = image.rgb_data[old_idx + 2];

            if let (Some(ref old_alpha), Some(ref mut new_alpha)) = (&image.alpha_data, &mut new_alpha
            ) {
                let old_a_idx = (y * image.width + x) as usize;
                let new_a_idx = (new_y * image.width + x) as usize;
                new_alpha[new_a_idx] = old_alpha[old_a_idx];
            }
        }
    }

    let mut result = if let Some(alpha) = new_alpha {
        ImageData::with_alpha(image.width, image.height, new_rgb, alpha)?
    } else {
        ImageData::new(image.width, image.height, new_rgb)?
    };

    result.color_space = image.color_space;
    result.source_path = image.source_path.clone();
    result.exif_orientation = Some(1);

    Ok(result)
}

/// Transpose image (swap x and y)
fn transpose(image: &ImageData) -> Result<ImageData> {
    // This is essentially a 90-degree rotation then flip
    rotate_90(image).and_then(|r| flip_horizontal(&r))
}

/// Transpose then flip
fn transpose_then_flip(image: &ImageData) -> Result<ImageData> {
    transpose(image).and_then(|t| flip_vertical(&t))
}

/// Rotate image by a specific angle (for PCA-based correction)
fn rotate_image(image: &ImageData, angle_degrees: f32) -> Result<ImageData> {
    // For now, only support standard rotations
    // Full arbitrary rotation would require interpolation
    let normalized = ((angle_degrees % 360.0) + 360.0) % 360.0;

    match normalized.round() as i32 {
        0 => Ok(image.clone()),
        90 | -270 => rotate_90(image),
        180 | -180 => rotate_180(image),
        270 | -90 => rotate_270(image),
        _ => {
            // Non-standard angle, return unchanged
            Ok(image.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let data: Vec<f32> = (0..width * height * 3)
            .map(|i| ((i % 256) as f32 / 255.0))
            .collect();
        ImageData::new(width, height, data).unwrap()
    }

    #[test]
    fn test_rotate_90() {
        let img = create_test_image(100, 200);
        let rotated = rotate_90(&img).unwrap();
        assert_eq!(rotated.width, 200);
        assert_eq!(rotated.height, 100);
    }

    #[test]
    fn test_rotate_180() {
        let img = create_test_image(100, 200);
        let rotated = rotate_180(&img).unwrap();
        assert_eq!(rotated.width, 100);
        assert_eq!(rotated.height, 200);
    }

    #[test]
    fn test_rotate_270() {
        let img = create_test_image(100, 200);
        let rotated = rotate_270(&img).unwrap();
        assert_eq!(rotated.width, 200);
        assert_eq!(rotated.height, 100);
    }

    #[test]
    fn test_flip_horizontal() {
        let img = create_test_image(100, 100);
        let flipped = flip_horizontal(&img).unwrap();
        assert_eq!(flipped.width, 100);
        assert_eq!(flipped.height, 100);
    }

    #[test]
    fn test_flip_vertical() {
        let img = create_test_image(100, 100);
        let flipped = flip_vertical(&img).unwrap();
        assert_eq!(flipped.width, 100);
        assert_eq!(flipped.height, 100);
    }

    #[test]
    fn test_normalization() {
        assert_eq!(normalize_to_standard_rotation(5.0), 0.0);
        assert_eq!(normalize_to_standard_rotation(85.0), 90.0);
        assert_eq!(normalize_to_standard_rotation(95.0), 90.0);
        assert_eq!(normalize_to_standard_rotation(175.0), 180.0);
        assert_eq!(normalize_to_standard_rotation(350.0), 0.0);
    }
}
