//! Orientation detection and correction

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
            // Try PCA first
            let (pca_orientation, confidence) = detect_orientation_pca(image)?;

            if confidence >= 0.6 {
                // Use PCA orientation
                let corrected = rotate_image(image, pca_orientation)?;
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
    }
}

/// Detect orientation using PCA principal axis
///
/// Returns (rotation_degrees, confidence)
fn detect_orientation_pca(image: &ImageData) -> Result<(f32, f32)> {
    use nalgebra::{DMatrix, SVD};

    // Get RGB data as a matrix of pixel positions weighted by intensity
    let (r, g, b) = image.split_channels();
    let n_pixels = image.num_pixels();

    // Calculate intensity-weighted pixel positions
    // For orientation, we care about where the "energy" is in the image
    let mut x_coords = Vec::with_capacity(n_pixels);
    let mut y_coords = Vec::with_capacity(n_pixels);
    let mut weights = Vec::with_capacity(n_pixels);

    for y in 0..image.height {
        for x in 0..image.width {
            let idx = (y * image.width + x) as usize;
            let intensity = (r[idx] + g[idx] + b[idx]) / 3.0;

            // Only include pixels with significant intensity
            if intensity > 0.1 {
                x_coords.push(x as f64);
                y_coords.push(y as f64);
                weights.push(intensity as f64);
            }
        }
    }

    if x_coords.len() < 100 {
        // Not enough data points
        return Ok((0.0, 0.0));
    }

    // Calculate weighted centroid
    let total_weight: f64 = weights.iter().sum();
    let mean_x: f64 = x_coords.iter().zip(&weights).map(|(x, w)| x * w).sum::<f64>() / total_weight;
    let mean_y: f64 = y_coords.iter().zip(&weights).map(|(y, w)| y * w).sum::<f64>() / total_weight;

    // Mean-center
    let centered_x: Vec<f64> = x_coords.iter().map(|x| x - mean_x).collect();
    let centered_y: Vec<f64> = y_coords.iter().map(|y| y - mean_y).collect();

    // Build covariance matrix
    let mut cov_xx = 0.0;
    let mut cov_yy = 0.0;
    let mut cov_xy = 0.0;

    for i in 0..centered_x.len() {
        let w = weights[i] / total_weight;
        cov_xx += centered_x[i] * centered_x[i] * w;
        cov_yy += centered_y[i] * centered_y[i] * w;
        cov_xy += centered_x[i] * centered_y[i] * w;
    }

    // Eigen-decomposition of 2x2 covariance
    let cov = DMatrix::from_row_slice(2, 2, &[
        cov_xx, cov_xy,
        cov_xy, cov_yy,
    ]);

    let svd = SVD::new(cov, true, true);
    let singular_values = svd.singular_values;

    // Calculate confidence from eigenvalue ratio
    let eigen_ratio = if singular_values[0] > 0.0 {
        singular_values[0] / (singular_values[0] + singular_values[1]).max(1e-10)
    } else {
        0.0
    };

    // Get principal axis direction
    let u = svd.u.expect("SVD U matrix should exist");
    let principal_x = u[(0, 0)];
    let principal_y = u[(1, 0)];

    // Calculate angle
    let angle_rad = principal_y.atan2(principal_x);
    let angle_deg = angle_rad.to_degrees();

    // Normalize to 0-360
    let normalized_angle = ((angle_deg % 360.0) + 360.0) % 360.0;

    // Determine which standard rotation this is closest to
    let rotation = normalize_to_standard_rotation(normalized_angle);

    // Confidence is based on eigenvalue ratio
    let confidence = (eigen_ratio * 2.0 - 1.0).clamp(0.0, 1.0);

    Ok((rotation, confidence))
}

/// Normalize angle to one of: 0, 90, 180, 270
fn normalize_to_standard_rotation(angle: f64) -> f32 {
    let rotations = [0.0f64, 90.0, 180.0, 270.0];
    let mut closest = 0.0f64;
    let mut min_diff = f64::MAX;

    for &rot in &rotations {
        let diff = (angle - rot).abs();
        let diff = diff.min(360.0 - diff); // Handle wrap-around
        if diff < min_diff {
            min_diff = diff;
            closest = rot;
        }
    }

    closest as f32
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
